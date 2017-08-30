#[macro_use] extern crate serenity;
extern crate dotenv;
extern crate websocket;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate futures;

mod commands;
mod lavalink;
mod handler;

use lavalink::rest::{self, HttpClient};
use lavalink::socket::Socket;

use std::env;
use std::thread;
use std::time::Duration;

use serenity::client::CACHE;
use serenity::framework::StandardFramework;
use serenity::model::*;
use serenity::prelude::*;
use serenity::voice;
use serenity::Result as SerenityResult;

use dotenv::dotenv;

use websocket::OwnedMessage;

use tokio_core::reactor::Core;

fn main() {
    // load .env into environment variables
    let _ = dotenv();

    // serenity stuff
    let token = env::var("DISCORD_TOKEN")
        .expect("erm lol wheres ur DISCORD_TOKEN");

    let mut client = Client::new(&token, handler::Handler);

    client.with_framework(StandardFramework::new()
        .configure(|c| c
            .prefix("!")
            .on_mention(true))
        .on("ping", commands::meta::ping));

    // lavalink stuff
    let mut core = Core::new().unwrap();

    // loading some tracks for fun!
    {
        let mut http_client = HttpClient::new(&mut core, "http://localhost:2333", "password");
        let request = http_client.create_request("/loadtracks?identifier=ytsearch:ncs%20my%20heart", None);
        let response = http_client.run_request(request);
        println!("response: {}", String::from_utf8_lossy(&response));
    }

    // lets create a new thread for lavalink to run on
    let lavalink_handle = thread::spawn(|| {
        let socket = Socket::open("ws://localhost:8012", "test-user-id", "password", 1);

        let messages = vec![
            OwnedMessage::Text("hey lol whats up".to_owned()),
            OwnedMessage::Ping(vec![]),
            OwnedMessage::Text("dab real hard".to_owned()),
        ];

        for message in messages {
            let copy_of_message = message.clone();

            thread::sleep(Duration::from_millis(5000));

            match socket.tx.send(message) {
                Ok(_) => { println!("sent {:?}", copy_of_message); },
                Err(e) => { println!("oh no! {:?}", e); }
            }
        }

        socket.close();
    });

    // start the discord client on the main thread
    //let _ = client.start()
    //    .map_err(|err| println!("serenity client ended: {:?}", err));

    let _ = lavalink_handle.join(); // wait for the lavalink thread to finish
}