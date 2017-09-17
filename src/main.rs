extern crate serenity;
extern crate dotenv;
extern crate websocket;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate futures;
extern crate percent_encoding;
extern crate typemap;
extern crate parking_lot;

mod commands;
mod lavalink;
mod handler;
mod keys;

use handler::GuildVoiceState;
use lavalink::config::Config;
use lavalink::socket::Socket;

use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};

use serenity::framework::StandardFramework;
use serenity::model::*;
use serenity::prelude::*;
use dotenv::dotenv;

fn main() {
    // load .env into environment variables
    let _ = dotenv();

    // getting environment variables
    let discord_token = env::var("DISCORD_TOKEN").unwrap();

    let lavalink_config = Config {
        http_host: env::var("LAVALINK_HTTP_HOST").unwrap(),
        websocket_host: env::var("LAVALINK_WEBSOCKET_HOST").unwrap(),
        user_id: env::var("LAVALINK_USER_ID").unwrap(),
        password: env::var("LAVALINK_PASSWORD").unwrap(),
        num_shards: env::var("LAVALINK_NUM_SHARDS")
            .map(|num_shards| num_shards.parse::<u64>().unwrap()).unwrap(),
    };

    // serenity!!!
    let mut client = Client::new(&discord_token, handler::Handler);

    // start the lavalink socket!!
    let lavalink_socket = Socket::open(&lavalink_config, client.shards.clone());

    client.with_framework(StandardFramework::new()
        .configure(|c| c
            .prefix("!")
            .on_mention(true))
        .on("stop", commands::admin::stop)
        .on("ping", commands::meta::ping)
        .on("join", commands::voice::join)
        .on("leave", commands::voice::leave)
        .on("search", commands::search::search)
        .on("play", commands::play::play));

    {
        let data = &mut client.data.lock();

        // add a clone of the lavalink config for the search command's http client
        let _ = data.insert::<keys::LavalinkConfig>(lavalink_config.clone());

        // add the close handle for the admin stop command to shutdown serenity
        let _ = data.insert::<keys::SerenityCloseHandle>(client.close_handle().clone());

        // add a clone of the socket sender as we cannot pass around lavalink_socket for #send
        let socket_sender = lavalink_socket.ws_tx.clone();
        let _ = data.insert::<keys::LavalinkSocketSender>(socket_sender);

        // map of guild voice states
        let voice_states: Mutex<HashMap<GuildId, Arc<Mutex<GuildVoiceState>>>> = Mutex::new(HashMap::new());
        let _ = data.insert::<keys::GuildVoiceState>(voice_states);
    }

    let _ = client.start()
        .map_err(|err| println!("serenity client ended: {:?}", err));

    // close the lavalink socket
    lavalink_socket.close();
}