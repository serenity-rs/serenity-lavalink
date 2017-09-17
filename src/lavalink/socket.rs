extern crate serde_json;

use super::config::Config;
use super::opcodes::*;
use super::stats::*;

use std::collections::HashMap;
use std::str::FromStr;
use std::sync;
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender, SendError};
use std::thread::{self, JoinHandle};

use parking_lot;
use websocket::{Message, OwnedMessage};
use websocket::client::ClientBuilder;
use websocket::header::Headers;
use serde_json::Value;
use serenity::gateway::Shard;

const WEBSOCKET_PROTOCOL: &'static str = "rust-websocket";

pub struct SocketState {
    pub stats: Option<RemoteStats>,
}

impl SocketState {
    fn new() -> Self {
        Self { stats: None, }
    }
}

pub struct Socket {
    pub ws_tx: Arc<sync::Mutex<Sender<OwnedMessage>>>,
    pub send_loop: JoinHandle<()>,
    pub recv_loop: JoinHandle<()>,
    pub state: Arc<sync::Mutex<SocketState>>,
}

impl Socket {
    pub fn open(config: &Config, shards: Arc<parking_lot::Mutex<HashMap<u64, Arc<parking_lot::Mutex<Shard>>>>>) -> Self {
        let mut headers = Headers::new();
        headers.set_raw("Authorization", vec![config.password.clone().as_bytes().to_vec()]);
        headers.set_raw("Num-Shards", vec![config.num_shards.to_string().as_bytes().to_vec()]);
        headers.set_raw("User-Id", vec![config.user_id.clone().as_bytes().to_vec()]);

        let client = ClientBuilder::new(config.websocket_host.clone().as_ref())
            .unwrap()
            .add_protocol(WEBSOCKET_PROTOCOL)
            .custom_headers(&headers)
            .connect_insecure()
            .unwrap();

        let (mut receiver, mut sender) = client.split().unwrap();

        let (ws_tx, ws_rx) = channel();
        let ws_tx_1 = ws_tx.clone();

        let state = Arc::new(sync::Mutex::new(SocketState::new()));

        let send_loop = thread::spawn(move || {
            loop {
                let message = match ws_rx.recv() {
                    Ok(m) => m,
                    Err(e) => {
                        println!("Send loop: {:?}", e);
                        return;
                    }
                };

                // handle close message, exit loop
                match message {
                    OwnedMessage::Close(_) => {
                        let _ = sender.send_message(&message);
                        return;
                    },
                    _ => (),
                }

                match sender.send_message(&message) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("Send loop: {:?}", e);
                        let _ = sender.send_message(&Message::close());
                        return;
                    }
                }
            }
        });

        let recv_state = state.clone(); // clone state for the recv loop otherwise ownership passed

        let recv_loop = thread::spawn(move || {
            for message in receiver.incoming_messages() {
                let message = match message {
                    Ok(m) => m,
                    Err(e) => {
                        println!("Receive loop: {:?}", e);
                        let _ = ws_tx_1.send(OwnedMessage::Close(None));
                        return;
                    }
                };

                match message {
                    // sever sent close msg, pass to send loop & break from loop
                    OwnedMessage::Close(_) => {
                        let _ = ws_tx_1.send(OwnedMessage::Close(None));
                        return;
                    },
                    OwnedMessage::Ping(data) => {
                        match ws_tx_1.send(OwnedMessage::Pong(data)) {
                            Ok(()) => (), // ponged well
                            Err(e) => {
                                // ponged badly and had an error, exit loop!?!>!?
                                println!("Receive loop: {:?}", e);
                                return;
                            }
                        }
                    },
                    OwnedMessage::Text(data) => {
                        let json: Value = serde_json::from_str(data.as_ref()).unwrap();
                        let op = json["op"].as_str().unwrap();
                        let opcode = Opcode::from_str(op).unwrap();

                        use super::opcodes::Opcode::*;

                        match opcode {
                            SendWS => {
                                let shard_id = json["shardId"].as_u64().unwrap();
                                let message = json["message"].as_str().unwrap();

                                let shards = &*shards.lock();
                                let shard = &mut *shards.get(&shard_id).unwrap().lock();

                                let _ = shard.client.send_message(&OwnedMessage::Text(message.to_owned()));
                            },
                            ValidationRequest => {
                                let guild_id_str = json["guildId"].as_str().unwrap();
                                let _guild_id_u64 = guild_id_str.parse::<u64>().unwrap();
                                let channel_id_str = json["channelId"].as_str();

                                // serenity inserts guilds into the cache once it becomes available
                                // so i need to wait for the guild to become available before
                                // initiating the connection
                                //
                                // this should not be an issue once connections are issued via
                                // commands as the command cannot be handled before the guild is
                                // available :)
                                //
                                // for testing i have set it to always return true as lavalink will
                                // continuously send validation requests and voice state updates
                                // until it has a voice server update anyway

                                /*let valid = match GuildId(guild_id_u64).find() {
                                    Some(_) => {
                                        if let Some(channel_id) = channel_id_str {
                                            let channel_id = ChannelId(channel_id.parse::<u64>().unwrap());
                                            channel_id.find().is_some()
                                        } else {
                                            true
                                        }
                                    },
                                    None => false,
                                };*/
                                let valid = true; // todo remove

                                let json = match channel_id_str {
                                    Some(channel_id) => {
                                        json!({
                                            "op": ValidationResponse.to_string(),
                                            "guildId": guild_id_str,
                                            "channelId": channel_id,
                                            "valid": valid,
                                        })
                                    },
                                    None => {
                                        json!({
                                            "op": ValidationResponse.to_string(),
                                            "guildId": guild_id_str,
                                            "valid": valid,
                                        })
                                    }
                                };

                                let _ = ws_tx_1.send(OwnedMessage::Text(json.to_string()));
                            },
                            IsConnectedRequest => {
                                let shard_id = json["shardId"].as_u64().unwrap();
                                let shards = &*shards.lock();

                                let json = json!({
                                    "op": IsConnectedResponse.to_string(),
                                    "shardId": shard_id,
                                    "connected": shards.contains_key(&shard_id),
                                });

                                let _ = ws_tx_1.send(OwnedMessage::Text(json.to_string()));
                            },
                            PlayerUpdate => {},
                            Stats => {
                                let stats = RemoteStats::from_json(&json);

                                let mut state = recv_state.lock().unwrap();
                                state.stats = Some(stats);
                            },
                            Event => {
                                let _guild_id = json["guildId"].as_str().unwrap();
                                let _track = json["track"].as_str().unwrap();

                                match json["type"].as_str().unwrap() {
                                    "TrackEndEvent" => {
                                        let _reason = json["reason"].as_str().unwrap();
                                    },
                                    "TrackExceptionEvent" => {
                                        let _error = json["error"].as_str().unwrap();
                                    },
                                    "TrackStuckEvent" => {
                                        let _threshold_ms = json["thresholdMs"].as_i64().unwrap();
                                    },
                                    unexpected => {
                                        println!("Unexpected event type: {}", unexpected)
                                    }
                                }
                            }
                            _ => {},
                        }
                    },
                    // probably wont happen
                    _ => {
                        println!("Receive loop: {:?}", message)
                    }
                }
            }
        });

        Self {
            ws_tx: Arc::new(sync::Mutex::new(ws_tx)),
            send_loop,
            recv_loop,
            state,
        }
    }

    pub fn send(&self, message: OwnedMessage) -> Result<(), SendError<OwnedMessage>> {
        let ws_tx = self.ws_tx.clone();
        let result = ws_tx.lock().unwrap().send(message);
        result
    }

    pub fn close(self) {
        println!("closing lavalink socket!");

        let _ = self.send(OwnedMessage::Close(None));

        let _ = self.send_loop.join();
        let _ = self.recv_loop.join();
    }
}