extern crate dotenv;
extern crate hyper;
extern crate parking_lot;
extern crate percent_encoding;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate serenity;
extern crate typemap;
extern crate websocket;

mod commands;
mod handler;
mod keys;
mod lavalink;

use std::env;

use dotenv::dotenv;
use lavalink::config::Config;
use lavalink::socket::Socket;
use serenity::framework::StandardFramework;
use serenity::framework::standard::help_commands;
use serenity::prelude::*;

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
            .prefix("RONNIEPICKERING")
            .on_mention(true)
            .allow_dm(false)
            .allow_whitespace(true)
            .ignore_bots(true))
        .group("admin", |g| g
            .command("shutdown", |c| c
                .exec(commands::admin::shutdown)
                .desc("turns the bot off")))
        .group("meta", |g| g
            .command("help", |c| c
                .exec_help(help_commands::with_embeds)
                .desc("shows a list of available commands"))
            .command("ping", |c| c
                .exec(commands::meta::ping)
                .desc("measures latency"))
            .command("stats", |c| c
                .exec(commands::meta::stats)
                .desc("shows lavalink node statistics")))
        .group("search", |g| g
            .command("search", |c| c
                .exec(commands::search::search)
                .desc("shows tracks for a search result")
                .example("search ytsearch:ncs mix")
                .usage("search <[prefix:]identifier>\nAvailable prefixes: ytsearch, scsearch")
                .min_args(1)))
        .group("voice", |g| g
            .command("join", |c| c
                .exec(commands::voice::join)
                .desc("joins a voice channel")
                .usage("1) join a voice channel\n2) use RONNIEPICKERING join\n3) :)"))
            .command("leave", |c| c
                .exec(commands::voice::leave)
                .desc("leaves a voice channel")))
        .group("audio", |g| g
            .command("play", |c| c
                .exec(commands::play::play)
                .desc("plays a track")
                .usage("play <base64 encoded track>")
                .min_args(1))
            .command("stop", |c| c
                .exec(commands::stop::stop)
                .desc("stops playing a track"))
            .command("pause", |c| c
                .exec(commands::pause::pause)
                .desc("pauses music playback"))
            .command("resume", |c| c
                .exec(commands::pause::resume)
                .desc("resumes music playback"))
            .command("volume", |c| c
                .exec(commands::volume::volume)
                .desc("changes the track volume")
                .example("volume 100")
                .usage("volume <1 - 150> (default: 100)")
                .min_args(1))
            .command("current", |c| c
                .exec(commands::current::current)
                .desc("shows the playing track"))));

    {
        let data = &mut client.data.lock();

        // add a clone of the lavalink config for the search command's http client
        let _ = data.insert::<keys::LavalinkConfig>(lavalink_config.clone());

        // add the close handle for the admin stop command to shutdown serenity
        let _ = data.insert::<keys::SerenityCloseHandle>(client.close_handle().clone());

        // add a clone of the socket sender as we cannot pass around lavalink_socket for #send
        let socket_sender = lavalink_socket.ws_tx.clone();
        let _ = data.insert::<keys::LavalinkSocketSender>(socket_sender);

        // add a clone of the socket state
        let socket_state = lavalink_socket.state.clone();
        let _ = data.insert::<keys::LavalinkSocketState>(socket_state);

        // lets gif the player manager :)
        let player_manager = lavalink_socket.player_manager.clone();
        let _ = data.insert::<keys::LavalinkAudioPlayerManager>(player_manager);
    }

    let _ = client.start()
        .map_err(|err| println!("serenity client ended: {:?}", err));

    // close the lavalink socket
    lavalink_socket.close();
}