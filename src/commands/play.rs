use keys;

use lavalink::opcodes::Opcode::Play;

use serenity::model::*;
use serenity::client::Context;
use serenity::framework::standard::Args;

use websocket::OwnedMessage;

pub fn play(ctx: &mut Context, msg: &Message, args: Args) -> Result<(), String> {
    let args = match args.list::<String>() {
        Ok(args) => args,
        Err(_) => {
            let _ = msg.channel_id.say("usage: !play <encoded track>");
            return Ok(());
        },
    };
    
    let data = ctx.data.lock();

    let ws_tx = data.get::<keys::LavalinkSocketSender>().unwrap().clone();

    let _ = ws_tx.lock().unwrap().send(OwnedMessage::Text(json!({
        "op": Play.to_string(),
        "guildId": "272410239947767808",
        "track": args.get(0),
    }).to_string()));

    Ok(())
}