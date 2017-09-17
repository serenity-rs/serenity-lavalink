use keys;

use lavalink::message;

use serenity::model::*;
use serenity::client::Context;
use serenity::framework::standard::Args;

pub fn play(ctx: &mut Context, msg: &Message, args: Args) -> Result<(), String> {
    let args = match args.list::<String>() {
        Ok(args) => args,
        Err(_) => {
            let _ = msg.channel_id.say("usage: !play <encoded track>");
            return Ok(());
        },
    };
    let track = args.get(0).unwrap();

    let guild_id = match msg.guild_id() {
        Some(guild_id) => guild_id.0.to_string(),
        None => {
            println!("oh no! no guild id??");
            return Ok(());
        },
    };
    
    let data = ctx.data.lock();
    let ws_tx = data.get::<keys::LavalinkSocketSender>().unwrap().clone();

    let _ = ws_tx.lock().unwrap().send(message::play(&guild_id, track));

    Ok(())
}

pub fn stop(ctx: &mut Context, msg: &Message, _: Args) -> Result<(), String> {
    let guild_id = match msg.guild_id() {
        Some(guild_id) => guild_id.0.to_string(),
        None => {
            println!("oh no! no guild id??");
            return Ok(());
        },
    };
    
    let data = ctx.data.lock();
    let ws_tx = data.get::<keys::LavalinkSocketSender>().unwrap().clone();

    let _ = msg.channel_id.say("stopping music! :)");

    let _ = ws_tx.lock().unwrap().send(message::stop(&guild_id));

    Ok(())
}