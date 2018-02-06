use parking_lot::{Mutex, RwLock};
use serde_json;
use serenity::client::bridge::gateway::{
    ShardClientMessage,
    ShardId,
    ShardRunnerMessage,
};
use serenity::gateway::InterMessage;
use std::net::TcpStream;
use std::str::FromStr;
use std::sync::mpsc::{self, Receiver as MpscReceiver, Sender as MpscSender};
use std::sync::Arc;
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
use websocket::receiver::Reader as WebSocketReader;
use websocket::sender::Writer as WebSocketWriter;
use websocket::{ClientBuilder, Message, OwnedMessage};
use lavalink::model::{IsConnectedResponse, ValidationResponse};
use lavalink::opcodes::Opcode;
use ::prelude::*;

#[derive(Debug)]
pub struct Node {
    pub websocket_host: String,
    pub sender: NodeSender,
    pub send_loop: JoinHandle<()>,
    pub recv_loop: JoinHandle<()>,
    pub state: NodeState,
}

impl Node {
    pub fn connect(config: &NodeConfig, shards: SerenityShardManager, player_manager: NodeAudioPlayerManager) -> Result<Self> {
        let mut headers = Headers::new();
        headers.set_raw("Authorization", vec![config.password.clone().as_bytes().to_vec()]);
        headers.set_raw("Num-Shards", vec![config.num_shards.to_string().as_bytes().to_vec()]);
        headers.set_raw("User-Id", vec![config.user_id.clone().as_bytes().to_vec()]);

        let client = ClientBuilder::new(config.websocket_host.clone().as_ref())?
            .add_protocol("rust-websocket")
            .custom_headers(&headers)
            .connect_insecure()?;

        let (mut receiver, mut sender) = client.split()?;

        let (ws_tx, mut ws_rx) = mpsc::channel();
        let ws_tx_1 = ws_tx.clone();

        let state = Arc::new(RwLock::new(State::new()));

        let builder = ThreadBuilder::new().name("send loop".into());
        let send_loop = builder.spawn(move || {
            send_loop(&mut ws_rx, &mut sender);
        }).unwrap();

        let recv_state = Arc::clone(&state); // clone state for the recv loop otherwise ownership passed

        //let player_manager_cloned = player_manager.clone(); // clone for move to recv loop

        let builder = ThreadBuilder::new().name("recv loop".into());
        let recv_loop = builder.spawn(move || {
            ReceiveLoop {
                player_manager: &player_manager,
                receiver: &mut receiver,
                recv_state: &recv_state,
                shards: &shards,
                ws_tx_1: &ws_tx_1,
            }.run();
        }).unwrap();

        Ok(Node {
            websocket_host: config.websocket_host.clone(),
            sender: Arc::new(Mutex::new(ws_tx)),
            send_loop,
            recv_loop,
            state,
        })
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

struct ReceiveLoop<'a> {
    receiver: &'a mut WebSocketReader<TcpStream>,
    ws_tx_1: &'a MpscSender<OwnedMessage>,
    shards: &'a SerenityShardManager,
    recv_state: &'a NodeState,
    player_manager: &'a NodeAudioPlayerManager,
}

impl<'a> ReceiveLoop<'a> {
    fn run(&mut self) {
        loop {
            let msg: OwnedMessage = match self.recv() {
                Ok(msg) => msg,
                Err(why) => {
                    error!("Error receiving msg: {:?}", why);
                    info!("Shutting down receive loop");
                    let _ = self.ws_tx_1.send(OwnedMessage::Close(None));

                    return;
                },
            };

            if !self.handle_message(msg) {
                return;
            }
        }
    }

    fn recv(&mut self) -> Result<OwnedMessage> {
        self.receiver.recv_message().map_err(From::from)
    }

    /// Handles the received message.
    ///
    /// Returns whether to continue the loop.
    fn handle_message(&self, msg: OwnedMessage) -> bool {
        match msg {
            OwnedMessage::Close(_) => {
                // sever sent close msg, pass to send loop & break from loop
                let _ = self.ws_tx_1.send(OwnedMessage::Close(None));

                return false;
            },
            OwnedMessage::Ping(data) => {
                if let Err(why) = self.ws_tx_1.send(OwnedMessage::Pong(data)) {
                    error!("Error ponging in receive loop: {:?}", why);

                    return false;
                }
            },
            OwnedMessage::Text(data) => {
                let json = match serde_json::from_str::<Value>(data.as_ref()) {
                    Ok(json) => json,
                    Err(why) => {
                        error!("Err parsing JSON in receive loop: {:?}", why);

                        return true;
                    },
                };

                let opcode = match json["op"].as_str() {
                    Some(opcode) => match Opcode::from_str(opcode) {
                        Ok(opcode) => opcode,
                        Err(why) => {
                            error!("Err parsing opcode: {:?}", why);

                            return true;
                        },
                    },
                    None => {
                        error!("Receive loop msg had no opcode; disregarding");

                        return true;
                    },
                };

                debug!("Receive loop msg with opcode: {:?}", &opcode);

                self.handle_opcode(json, &opcode);
            },
            // probably wont happen
            _ => {
                debug!("Receive loop: {:?}", msg)
            },
        }

        true
    }

    fn handle_opcode(&self, json: Value, opcode: &Opcode) {
        use self::Opcode::*;

        match *opcode {
            SendWS => self.handle_send_ws(&json),
            ValidationReq => self.handle_validation_request(&json),
            IsConnectedReq => self.handle_is_connected_request(&json),
            PlayerUpdate => self.handle_player_update(&json),
            Stats => self.handle_state(json),
            Event => self.handle_event(&json),
            _ => return,
        }
    }

    fn handle_event(&self, json: &Value) {

        let guild_id_str = json["guildId"]
            .as_str()
            .expect("invalid json guildId - should be str");
        let guild_id = guild_id_str
            .parse::<u64>()
            .expect("could not parse json guild_id into u64");
        let track = json["track"]
            .as_str()
            .expect("invalid json track - should be str");

        let player_manager = self.player_manager.read();

        let player = match player_manager.get_player(&guild_id) {
            Some(player) => player,
            None => {
                warn!(
                    "got invalid audio player update for guild {:?}",
                    guild_id,
                );

                return;
            }
        };

        let mut player = player.lock();

        match json["type"].as_str().expect("Err parsing type to str") {
            "TrackEndEvent" => {
                let reason = json["reason"]
                    .as_str()
                    .expect("invalid json reason - should be str");

                // Set the player's track so nothing is playing, reset
                // the time, and reset the position
                player.track = None;
                player.time = 0;
                player.position = 0;

                self.player_manager.read().listener.track_end(&mut player, track, reason);
            },
            "TrackExceptionEvent" => {
                let error = json["error"]
                    .as_str()
                    .expect("invalid json error - should be str");

                // TODO: determine if should keep playing

                self.player_manager.read().listener.track_exception(&mut player, track, error);
            },
            "TrackStuckEvent" => {
                let threshold_ms = json["thresholdMs"]
                    .as_i64()
                    .expect("invalid json thresholdMs - should be i64");

                self.player_manager.read().listener.track_stuck(&mut player, track, threshold_ms);
            },
            other => {
                warn!("Unexpected event type: {}", other);
            },
        }
    }

    fn handle_is_connected_request(&self, json: &Value) {
        let shard_id = json["shardId"]
            .as_u64()
            .expect("invalid json shardId - should be u64");
        let shards = self.shards.lock();

        let msg = serde_json::to_vec(&IsConnectedResponse::new(
            shard_id,
            shards.has(ShardId(shard_id)),
        ));

        if let Ok(msg) = msg {
            let _ = self.ws_tx_1.send(OwnedMessage::Binary(msg));
        }
    }

    fn handle_player_update(&self, json: &Value) {
        let guild_id_str = json["guildId"]
            .as_str()
            .expect("expected json guildId - should be str");
        let guild_id = guild_id_str
            .parse::<u64>()
            .expect("could not parse json guild_id into u64");
        let state = json["state"]
            .as_object()
            .expect("json does not contain state object");
        let time = state["time"]
            .as_i64()
            .expect("json state has no time; should be i64");
        let position = state["position"]
            .as_i64()
            .expect("json state has no position; should be i64");

        let player_manager = self.player_manager.read();

        let player = match player_manager.get_player(&guild_id) {
            Some(player) => player,
            None => {
                warn!(
                    "got invalid audio player update for guild {:?}",
                    guild_id,
                );

                return;
            },
        };

        let mut player = player.lock();
        player.time = time;
        player.position = position;
    }

    fn handle_send_ws(&self, json: &Value) {
        let shard_id = json["shardId"]
            .as_u64()
            .expect("invalid json shardId - should be u64");
        let message = json["message"]
            .as_str()
            .expect("invalid json message - should be str");

        let shards = self.shards.lock();
        let mut runners = shards.runners.lock();

        if let Some(shard) = runners.get_mut(&ShardId(shard_id)) {
            let text = OwnedMessage::Text(message.to_owned());
            let client_msg = ShardClientMessage::Runner(
                ShardRunnerMessage::Message(text)
            );
            let msg = InterMessage::Client(client_msg);
            let _ = shard.runner_tx.send(msg);
        }
    }

    fn handle_state(&self, json: Value) {
        match serde_json::from_value(json) {
            Ok(stats) => {
                let mut state = self.recv_state.write();
                state.stats = Some(stats);
            },
            Err(e) => println!("Error parsing stats! {:?}", e),
        }
    }

    fn handle_validation_request(&self, json: &Value) {
        let guild_id_str = json["guildId"]
            .as_str()
            .expect("invalid json guildId - should be str");
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

        let msg = serde_json::to_vec(&ValidationResponse::new(
            guild_id_str,
            channel_id_str,
            valid,
        ));

        if let Ok(msg) = msg {
            let _ = self.ws_tx_1.send(OwnedMessage::Binary(msg));
        }
    }
}

fn send_loop(
    ws_rx: &mut MpscReceiver<OwnedMessage>,
    sender: &mut WebSocketWriter<TcpStream>,
) {
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
}
