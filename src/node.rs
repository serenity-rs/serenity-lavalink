use super::message;
use super::opcodes::*;
use super::player::*;
use super::stats::*;

use parking_lot;
use serde_json;
use serenity::gateway::Shard;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::mpsc::{channel, Sender};
use std::thread::{self, JoinHandle};
use websocket::client::ClientBuilder;
use websocket::header::Headers;
use websocket::{Message, OwnedMessage};
use ::prelude::*;

pub type NodeAudioPlayerManager = Arc<RwLock<AudioPlayerManager>>;
pub type NodeSender = Arc<Mutex<Sender<OwnedMessage>>>;
pub type NodeState = Arc<RwLock<State>>;

pub type SerenityShardMap = Arc<parking_lot::Mutex<HashMap<u64, Arc<parking_lot::Mutex<Shard>>>>>;

#[derive(Clone, Debug)]
pub struct NodeConfig {
    pub http_host: String,
    pub websocket_host: String,
    pub user_id: String,
    pub password: String,
    pub num_shards: u64,
}

#[derive(Clone, Debug, Default)]
pub struct State {
    pub stats: Option<RemoteStats>,
}

impl State {
    fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug)]
pub struct Node {
    pub websocket_host: String,
    pub sender: NodeSender,
    pub send_loop: JoinHandle<()>,
    pub recv_loop: JoinHandle<()>,
    pub state: NodeState,
}

impl Node {
    pub fn connect(config: &NodeConfig, shards: SerenityShardMap, player_manager: NodeAudioPlayerManager) -> Self {
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

        let (ws_tx, ws_rx) = channel();
        let ws_tx_1 = ws_tx.clone();

        let state = Arc::new(RwLock::new(State::new()));

        let builder = thread::Builder::new().name("send loop".into());
        let send_loop = builder.spawn(move || {
            loop {
                let message = match ws_rx.recv() {
                    Ok(m) => m,
                    Err(e) => {
                        println!("Send loop: {:?}", e);
                        return;
                    },
                };

                // handle close message, exit loop
                if let OwnedMessage::Close(_) = message {
                    let _ = sender.send_message(&message);
                    return;
                }

                if let Err(e) = sender.send_message(&message) {
                    println!("Send loop: {:?}", e);
                    let _ = sender.send_message(&Message::close());
                    return;
                }
            }
        }).unwrap();

        let recv_state = Arc::clone(&state); // clone state for the recv loop otherwise ownership passed

        //let player_manager_cloned = player_manager.clone(); // clone for move to recv loop

        let builder = thread::Builder::new().name("recv loop".into());
        let recv_loop = builder.spawn(move || {
            for message in receiver.incoming_messages() {
                let message = match message {
                    Ok(m) => m,
                    Err(e) => {
                        println!("Receive loop: {:?}", e);
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
                                println!("Receive loop: {:?}", e);
                                return;
                            },
                        }
                    },
                    OwnedMessage::Text(data) => {
                        let json: Value = match serde_json::from_str(data.as_ref()) {
                            Ok(json) => json,
                            Err(e) => {
                                println!("could not parse json {:?}", e);
                                continue;
                            },
                        };

                        let opcode = match json["op"].as_str() {
                            Some(opcode) => match Opcode::from_str(opcode) {
                                Ok(opcode) => opcode,
                                Err(e) => {
                                    println!("could not parse json opcode {:?}", e);
                                    continue;
                                },
                            },
                            None => {
                                println!("json did not include opcode - disgarding message");
                                continue;
                            },
                        };

                        use super::opcodes::Opcode::*;

                        match opcode {
                            SendWS => {
                                let shard_id = json["shardId"].as_u64().expect("invalid json shardId - should be u64");
                                let message = json["message"].as_str().expect("invalid json message - should be str");

                                let shards = &*shards.lock();
                                let shard = &mut *shards.get(&shard_id).unwrap().lock();

                                //let _ = shard.client.send_message(&Evzht9h3nznqzwlMessage::text(message.to_owned()));
                                let _ = shard.client.send_message(&OwnedMessage::Text(message.to_owned()));
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
                                    valid
                                ));
                            },
                            IsConnectedReq => {
                                let shard_id = json["shardId"].as_u64().expect("invalid json shardId - should be u64");
                                let shards = &*shards.lock();

                                let _ = ws_tx_1.send(message::is_connected_response(
                                    shard_id,
                                    shards.contains_key(&shard_id)
                                ));
                            },
                            PlayerUpdate => {
                                let guild_id_str = json["guildId"].as_str().expect("expected json guildId - should be str");
                                let guild_id = guild_id_str.parse::<u64>().expect("could not parse json guild_id into u64");
                                let state = json["state"].as_object().expect("json does not contain state object");
                                let time = state["time"].as_i64().expect("json state object does not contain time - should be i64");
                                let position = state["position"].as_i64().expect("json state object does not contain position - should be i64");

                                let player_manager = player_manager.read().expect("could not get access to player_manager mutex"); // unlock the mutex

                                let player = match player_manager.get_player(&guild_id) {
                                    Some(player) => player, // returns already cloned Arc
                                    None => {
                                        println!("got invalid audio player update for guild {:?}", &guild_id);
                                        continue;
                                    },
                                };

                                let mut player = player.lock().expect("could not get access to player mutex"); // unlock the player mutex
                                player.time = time;
                                player.position = position;
                            },
                            Stats => {
                                let stats = RemoteStats::from_json(&json);

                                let mut state = recv_state.write().expect("could not get write lock on recv state");
                                state.stats = Some(stats);
                            },
                            Event => {
                                let guild_id_str = json["guildId"].as_str().expect("invalid json guildId - should be str");
                                let guild_id = guild_id_str.parse::<u64>().expect("could not parse json guild_id into u64");
                                let track = json["track"].as_str().expect("invalid json track - should be str");

                                let player_manager = player_manager.read().expect("could not get access to player_manager mutex"); // unlock the mutex

                                let player = match player_manager.get_player(&guild_id) {
                                    Some(player) => player, // returns already cloned Arc
                                    None => {
                                        println!("got invalid audio player update for guild {:?}", &guild_id);
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
                                    unexpected => {
                                        println!("Unexpected event type: {}", unexpected);
                                    },
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
        self.sender.lock()
            .expect("could not get access to ws_tx mutex")
            .send(message)
            .map_err(From::from)
    }

    pub fn close(self) {
        println!("closing lavalink socket!");

        let _ = self.send(OwnedMessage::Close(None));

        let _ = self.send_loop.join();
        let _ = self.recv_loop.join();
    }
}

#[derive(Clone, Debug, Default)]
pub struct NodeManager {
    pub nodes: Arc<RwLock<Vec<Arc<Node>>>>,
    pub player_manager: NodeAudioPlayerManager,
}

impl NodeManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, config: &NodeConfig, shards: SerenityShardMap) {
        let node = Node::connect(config, shards, Arc::clone(&self.player_manager));

        let mut nodes = self.nodes.write()
            .expect("could not get write lock on nodes");

        nodes.push(Arc::new(node));
    }

    pub fn determine_best_node(&self) -> Option<Arc<Node>> {
        let nodes = self.nodes.read()
            .expect("could not get read lock on nodes");

        let mut record = i32::max_value();
        let mut best = None;

        for node in nodes.iter() {
            let total = Self::get_penalty(node).unwrap_or(0);

            if total < record {
                best = Some(Arc::clone(node));
                record = total;
            }
        }

        best
    }

    pub fn get_penalty(node: &Arc<Node>) -> Result<i32> {
        let state = node.state.read().expect("could not get read lock on node state");

        let stats = match state.stats.clone() {
            Some(stats) => stats,
            None => return Err(Error::StatsNotPresent),
        };

        let cpu = 1.05f64.powf(100f64 * stats.system_load) * 10f64 - 10f64;

        let (deficit_frame, null_frame) = match stats.frame_stats {
            Some(frame_stats) => {
                (
                    1.03f64.powf(500f64 * (f64::from(frame_stats.deficit) / 3000f64)) * 300f64 - 300f64,
                    (1.03f64.powf(500f64 * (f64::from(frame_stats.nulled) / 3000f64)) * 300f64 - 300f64) * 2f64,
                )
            },
            None => (0f64, 0f64),
        };

        Ok(stats.playing_players + cpu as i32 + deficit_frame as i32 + null_frame as i32)
    }

    pub fn close(self) {
        let nodes = if let Ok(nodes) = Arc::try_unwrap(self.nodes) {
            nodes
        } else {
            panic!("could not Arc::try_unwrap self.nodes");
        };

        let nodes = nodes.into_inner().expect("could not get rwlock inner for nodes");

        for node in nodes {
            let node = match Arc::try_unwrap(node) {
                Ok(node) => node,
                Err(_) => {
                    println!("could not Arc::try_unwrap node");
                    continue;
                }
            };

            node.close();
        }
    }
}
