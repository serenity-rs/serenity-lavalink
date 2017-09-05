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