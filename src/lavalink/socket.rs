extern crate serde_json;

use super::opcodes::*;
use super::stats::*;

use std::thread::{self, Thread, JoinHandle};
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

pub struct Socket {
    pub tx: Sender<OwnedMessage>,
    pub send_loop: JoinHandle<()>,
    pub recv_loop: JoinHandle<()>,
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

        // create a clone because thread::spawn(move... takes ownership of tx & we want to send
        // into the channel from multiple threads
        let tx_1 = tx.clone();

        // tbis loop waits for a new message in the rx channel. once the message has been received
        // it sends it to the websocket via the client's sender
        let send_loop = thread::spawn(move || {
            loop {
                // send loop
                let message = match rx.recv() {
                    // blocking call, waits for a message in rx channel
                    Ok(m) => m,
                    Err(e) => {
                        println!("Send loop: {:?}", e);
                        return;
                    }
                };

                match message {
                    // if it is a close message send the message and then return; to exit the loop
                    OwnedMessage::Close(_) => {
                        let _ = sender.send_message(&message);
                        return;
                    },
                    // otherwise continue
                    _ => (),
                }

                // send the message
                match sender.send_message(&message) {
                    Ok(()) => (), // message was sent successfully
                    Err(e) => {
                        // oh no an error occurred when sending the message
                        println!("Send loop: {:?}", e);
                        let _ = sender.send_message(&Message::close());
                        return;
                    }
                }
            }
        });

        // this loop waits for a new message to from the websocket server
        let recv_loop = thread::spawn(move || {
            // receiver.incoming_messages() is a blocking call that waits for the next message
            for message in receiver.incoming_messages() {
                let message = match message {
                    // woo message came in and its not broken!!
                    Ok(m) => m,
                    // oopsie that message is fucked lmao, send a close message into the sending
                    // channel and then exit out of the loop to stop the thread execution
                    Err(e) => {
                        println!("Receive loop: {:?}", e);
                        let _ = tx_1.send(OwnedMessage::Close(None));
                        return;
                    }
                };

                match message {
                    // the server sent a close message so send a close message into the sending
                    // channel to kill the sending thread, then return to exit out of the loop and
                    // stop the thread execution
                    OwnedMessage::Close(_) => {
                        let _ = tx_1.send(OwnedMessage::Close(None));
                        return;
                    },
                    // hehe the server sent a ping :) lets send a pong to the sending channel in
                    // response my dude
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
                    // text msg!!!!!!!!
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
                                println!("Stats = {:?}", stats);
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
                    // received something else?
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
        }
    }

    pub fn close(self) {
        let _ = self.tx.send(OwnedMessage::Close(None));

        let _ = self.send_loop.join();
        let _ = self.recv_loop.join();
    }
}