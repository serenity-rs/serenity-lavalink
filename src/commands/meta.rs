use keys;

use lavalink::stats::RemoteStats;

use serenity::model::*;
use serenity::client::Context;
use serenity::framework::standard::Args;

pub fn ping(ctx: &mut Context, msg: &Message, _: Args) -> Result<(), String> {
    if let Some(latency) = ctx.shard.lock().latency() {
        let _ = msg.channel_id.say(format!("Pong! Shard gateway heartbeat latency: {}.{}s",
                                           latency.as_secs(), latency.subsec_nanos()));
    } else {
        let _ = msg.channel_id.say("Pong! No latency recorded!");
    }

    Ok(())
}

fn get_socket_stats(ctx: &mut Context) -> Result<RemoteStats, &'static str> {
    let data = ctx.data.lock();

    let socket_state = match data.get::<keys::LavalinkSocketState>() {
        Some(socket_state) => socket_state,
        None => return Err("keys::LavalinkSocketState is not present in Context::data"),
    };
    
    let socket_state = socket_state.lock().expect("could not get lock on socket state");

    match socket_state.stats.clone() {
        Some(stats) => Ok(stats),
        None => {
            Err("no socket stats are available yet")
        }
    }
}

pub fn stats(ctx: &mut Context, msg: &Message, _: Args) -> Result<(), String> {
    let stats = get_socket_stats(ctx);

    let _ = msg.channel_id.say(
        &format!("lavalink node:```\n{:?}\n```", &stats)
    );

    Ok(())
}