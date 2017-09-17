use keys;

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

pub fn stats(ctx: &mut Context, msg: &Message, _: Args) -> Result<(), String> {
    let data = ctx.data.lock();

    let socket_state = data.get::<keys::LavalinkSocketState>().unwrap();
    let socket_state = socket_state.lock().unwrap();

    let stats = match socket_state.stats.clone() {
        Some(stats) => stats,
        None => {
            let _ = msg.channel_id.say("sry lol no node stats available");
            return Ok(());
        },
    };
    
    let mut response = String::new();
    response.push_str(&format!("lavalink node:```\n{:?}\n```", &stats));

    let _ = msg.channel_id.say(response);

    Ok(())
}