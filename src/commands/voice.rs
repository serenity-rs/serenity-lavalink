use keys;
use lavalink::opcodes::Opcode::Connect;

use serenity::model::*;
use serenity::client::Context;
use serenity::framework::standard::Args;
use websocket::OwnedMessage;

pub fn join(ctx: &mut Context, msg: &Message, _: Args) -> Result<(), String> {
    let _ = msg.channel_id.say("joining channel");

    let data = ctx.data.lock();

    let ws_tx = data.get::<keys::LavalinkSocketSender>().unwrap().clone();

    let _ = ws_tx.lock().unwrap().send(OwnedMessage::Text(json!({
        "op": Connect.to_string(),
        "guildId": "272410239947767808",
        "channelId": "320643590986399749",
    }).to_string()));

    Ok(())
}

pub fn leave(_: &mut Context, _: &Message, _: Args) -> Result<(), String> {
    Ok(())
}