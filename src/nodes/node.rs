use ::message;

use parking_lot::{Mutex, RwLock};
use serde_json;
use serenity::client::bridge::gateway::{
    ShardClientMessage,
    ShardId,
    ShardRunnerMessage,
};
use std::str::FromStr;
use std::sync::{Arc, mpsc};
use std::thread::{Builder as ThreadBuilder, JoinHandle};
use super::{
    NodeAudioPlayerManager,
    NodeConfig,
    NodeSender,
    NodeState,
    SerenityShardManager,
    State,
};
use websocket::header::Headers;
use websocket::{ClientBuilder, Message, OwnedMessage};
use ::opcodes::Opcode;
use ::prelude::*;
use ::stats::RemoteStats;

#[derive(Debug)]
pub struct Node {
    pub websocket_host: String,
    pub sender: NodeSender,
    pub send_loop: JoinHandle<()>,
    pub recv_loop: JoinHandle<()>,
    pub state: NodeState,
}

impl Node {
    pub fn connect(config: &NodeConfig, shards: SerenityShardManager, player_manager: NodeAudioPlayerManager) -> Self {
        let mut headers = Headers::new();
        headers.set_raw("Authorization", vec![config.password.clone().as_bytes().to_vec()]);
        headers.set_raw("Num-Shards", vec![config.num_shards.to_string().as_bytes().to_vec()]);
        headers.set_raw("User-Id", vec![config.user_id.clone().as_bytes().to_vec()]);

        let client = ClientBuilder::new(config.websocket_host.clone().as_ref())
            .unwrap()
            .add_protocol("rust-websocket")
            .custom_headers(&headers)
            .connect_insecure()
            .unwrap();

        let (mut receiver, mut sender) = client.split().unwrap();

        let (ws_tx, ws_rx) = mpsc::channel();
        let ws_tx_1 = ws_tx.clone();

        let state = Arc::new(RwLock::new(State::new()));

        let builder = ThreadBuilder::new().name("send loop".into());
        let send_loop = builder.spawn(move || {
            loop {
                let message = match ws_rx.recv() {
                    Ok(m) => m,
                    Err(e) => {
                        error!("Send loop: {:?}", e);
                        return;
                    },
                };

                // handle close message, exit loop
                if let OwnedMessage::Close(_) = message {
                    let _ = sender.send_message(&message);
                    return;
                }

                if let Err(e) = sender.send_message(&message) {
                    error!("Send loop: {:?}", e);
                    let _ = sender.send_message(&Message::close());
                    return;
                }
            }
        }).unwrap();

        let recv_state = Arc::clone(&state); // clone state for the recv loop otherwise ownership passed

        //let player_manager_cloned = player_manager.clone(); // clone for move to recv loop

        let builder = ThreadBuilder::new().name("recv loop".into());
        let recv_loop = builder.spawn(move || {
            for message in receiver.incoming_messages() {
                let message = match message {
                    Ok(m) => m,
                    Err(e) => {
                        error!("Receive loop: {:?}", e);
                        let _ = ws_tx_1.send(OwnedMessage::Close(None));
                        return;
                    },
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
                                error!("Receive loop: {:?}", e);
                                return;
                            },
                        }
                    },
                    OwnedMessage::Text(data) => {
                        let json: Value = match serde_json::from_str(data.as_ref()) {
                            Ok(json) => json,
                            Err(e) => {
                                error!("could not parse json {:?}", e);
                                continue;
                            },
                        };

                        let opcode = match json["op"].as_str() {
                            Some(opcode) => match Opcode::from_str(opcode) {
                                Ok(opcode) => opcode,
                                Err(e) => {
                                    error!("could not parse json opcode {:?}", e);
                                    continue;
                                },
                            },
                            None => {
                                error!("json did not include opcode - disgarding message");
                                continue;
                            },
                        };

                        use self::Opcode::*;

                        match opcode {
                            SendWS => {
                                let shard_id = json["shardId"].as_u64().expect("invalid json shardId - should be u64");
                                let message = json["message"].as_str().expect("invalid json message - should be str");

                                let shards = shards.lock();
                                let mut runners = shards.runners.lock();
                                let mut shard = runners.get_mut(&ShardId(shard_id)).unwrap();

                                let msg = ShardClientMessage::Runner(
                                    ShardRunnerMessage::Message(OwnedMessage::Text(message.to_owned()))
                                );
                                let _ = shard.runner_tx.send(msg);
                            },
                            ValidationReq => {
                                let guild_id_str = json["guildId"].as_str().expect("invalid json guildId - should be str");
                                let _guild_id_u64 = guild_id_str.parse::<u64>().expect("could not parse json guildId as u64");
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

                                let _ = ws_tx_1.send(message::validation_response(
                                    guild_id_str,
                                    channel_id_str,
                                    valid,
                                ));
                            },
                            IsConnectedReq => {
                                let shard_id = json["shardId"].as_u64().expect("invalid json shardId - should be u64");
                                let shards = shards.lock();

                                let _ = ws_tx_1.send(message::is_connected_response(
                                    shard_id,
                                    shards.has(ShardId(shard_id)),
                                ));
                            },
                            PlayerUpdate => {
                                let guild_id_str = json["guildId"].as_str().expect("expected json guildId - should be str");
                                let guild_id = guild_id_str.parse::<u64>().expect("could not parse json guild_id into u64");
                                let state = json["state"].as_object().expect("json does not contain state object");
                                let time = state["time"].as_i64().expect("json state object does not contain time - should be i64");
                                let position = state["position"].as_i64().expect("json state object does not contain position - should be i64");

                                let player_manager = player_manager.read(); // unlock the mutex

                                let player = match player_manager.get_player(&guild_id) {
                                    Some(player) => player, // returns already cloned Arc
                                    None => {
                                        warn!("got invalid audio player update for guild {:?}", &guild_id);
                                        continue;
                                    },
                                };

                                let mut player = player.lock().expect("could not get access to player mutex"); // unlock the player mutex
                                player.time = time;
                                player.position = position;
                            },
                            Stats => {
                                let stats = RemoteStats::from_json(&json);

                                let mut state = recv_state.write();
                                state.stats = Some(stats);
                            },
                            Event => {
                                let guild_id_str = json["guildId"].as_str().expect("invalid json guildId - should be str");
                                let guild_id = guild_id_str.parse::<u64>().expect("could not parse json guild_id into u64");
                                let track = json["track"].as_str().expect("invalid json track - should be str");

                                let player_manager = player_manager.read(); // unlock the mutex

                                let player = match player_manager.get_player(&guild_id) {
                                    Some(player) => player, // returns already cloned Arc
                                    None => {
                                        warn!("got invalid audio player update for guild {:?}", &guild_id);
                                        continue;
                                    }
                                };

                                let mut player = player.lock().expect("could not get access to player mutex"); // unlock the player mutex

                                match json["type"].as_str().unwrap() {
                                    "TrackEndEvent" => {
                                        let reason = json["reason"].as_str().expect("invalid json reason - should be str");

                                        player.track = None; // set track to None so nothing is playing
                                        player.time = 0; // reset the time
                                        player.position = 0; // reset the position

                                        for listener in &player.listeners {
                                            let on_track_end = &listener.on_track_end;
                                            on_track_end(&player, track, reason);
                                        }
                                    },
                                    "TrackExceptionEvent" => {
                                        let error = json["error"].as_str().expect("invalid json error - should be str");

                                        // todo determine if should keep playing

                                        for listener in &player.listeners {
                                            let on_track_exception = &listener.on_track_exception;
                                            on_track_exception(&player, track, error);
                                        }
                                    },
                                    "TrackStuckEvent" => {
                                        let threshold_ms = json["thresholdMs"].as_i64().expect("invalid json thresholdMs - should be i64");

                                        for listener in &player.listeners {
                                            let on_track_stuck = &listener.on_track_stuck;
                                            on_track_stuck(&player, track, threshold_ms);
                                        }
                                    },
                                    other => {
                                        warn!("Unexpected event type: {}", other);
                                    },
                                }
                            },
                            _ => {},
                        }
                    },
                    // probably wont happen
                    _ => {
                        debug!("Receive loop: {:?}", message)
                    },
                }
            }
        }).unwrap();

        Node {
            websocket_host: config.websocket_host.clone(),
            sender: Arc::new(Mutex::new(ws_tx)),
            send_loop,
            recv_loop,
            state,
        }
    }

    pub fn send(&self, message: OwnedMessage) -> Result<()> {
        self.sender.lock().send(message).map_err(From::from)
    }

    pub fn close(self) {
        info!("closing lavalink socket!");

        let _ = self.send(OwnedMessage::Close(None));
        let _ = self.send_loop.join();
        let _ = self.recv_loop.join();
    }
}
