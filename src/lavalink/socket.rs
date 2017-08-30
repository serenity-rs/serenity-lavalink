extern crate serde_json;

use super::opcodes::*;
use super::stats::*;

use std::thread::{self, Thread, JoinHandle};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::io::stdin;
use std::str::FromStr;
use std::collections::HashMap;
use std::rc::Rc;

use websocket::{Message, OwnedMessage};
use websocket::client::ClientBuilder;
use websocket::header::Headers;
use serde_json::{Value, Error};

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
    pub tx: Sender<OwnedMessage>,
    pub send_loop: JoinHandle<()>,
    pub recv_loop: JoinHandle<()>,
    pub state: Arc<Mutex<SocketState>>,
}

impl Socket {
    pub fn open(socket_uri: &str, user_id: &str, password: &str, num_shards: i32) -> Self {
        let mut headers = Headers::new();
        headers.set_raw("Authorization", vec![password.as_bytes().to_vec()]);
        headers.set_raw("Num-Shards", vec![num_shards.to_string().as_bytes().to_vec()]);
        headers.set_raw("User-Id", vec![user_id.as_bytes().to_vec()]);

        let client = ClientBuilder::new(socket_uri.as_ref())
            .unwrap()
            .add_protocol(WEBSOCKET_PROTOCOL)
            .custom_headers(&headers)
            .connect_insecure()
            .unwrap();

        let (mut receiver, mut sender) = client.split().unwrap();

        let (tx, rx) = channel();
        let tx_1 = tx.clone();

        let state = Arc::new(Mutex::new(SocketState::new()));

        let send_loop = thread::spawn(move || {
            loop {
                let message = match rx.recv() {
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
                        let _ = tx_1.send(OwnedMessage::Close(None));
                        return;
                    }
                };

                match message {
                    // sever sent close msg, pass to send loop & break from loop
                    OwnedMessage::Close(_) => {
                        let _ = tx_1.send(OwnedMessage::Close(None));
                        return;
                    },
                    OwnedMessage::Ping(data) => {
                        match tx_1.send(OwnedMessage::Pong(data)) {
                            Ok(()) => (), // ponged well
                            Err(e) => {
                                // ponged badly and had an error, exit loop!?!>!?
                                println!("Receive loop: {:?}", e);
                                return;
                            }
                        }
                    },
                    OwnedMessage::Text(data) => {
                        println!("Receive loop text message: {}", data);

                        let json: Value = serde_json::from_str(data.as_ref()).unwrap();
                        let op = json["op"].as_str().unwrap();
                        let opcode = Opcode::from_str(op).unwrap();

                        use super::opcodes::Opcode::*;

                        match opcode {
                            SendWS => {},
                            ValidationRequest => {},
                            IsConnectedRequest => {},
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

                                // todo get Player by guild_id & send event to PlayerListener
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
            tx,
            send_loop,
            recv_loop,
            state,
        }
    }

    pub fn close(self) {
        let _ = self.tx.send(OwnedMessage::Close(None));

        let _ = self.send_loop.join();
        let _ = self.recv_loop.join();
    }
}