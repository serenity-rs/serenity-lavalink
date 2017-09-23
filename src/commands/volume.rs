use keys;

use lavalink::message;
use serenity::client::Context;
use serenity::framework::standard::Args;
use serenity::model::*;

const INVALID_ARGUMENTS_MESSAGE: &'static str = "usage: `!volume <1 to 150>` (default 100)";

pub fn volume(ctx: &mut Context, msg: &Message, args: Args) -> Result<(), String> {
    let volume = match args.list::<String>() {
        Ok(args) => {
            if args.len() == 0 {
                let _ = msg.channel_id.say(INVALID_ARGUMENTS_MESSAGE);
                return Ok(());
            }

            match args.get(0).unwrap().parse::<i32>() {
                Ok(volume) => volume,
                Err(_) => {
                    let _ = msg.channel_id.say(INVALID_ARGUMENTS_MESSAGE);
                    return Ok(());
                }
            }
        },
        Err(_) => {
            let _ = msg.channel_id.say(INVALID_ARGUMENTS_MESSAGE);
            return Ok(());
        },
    };

    if volume < 1 || volume > 150 {
        let _ = msg.channel_id.say(INVALID_ARGUMENTS_MESSAGE);
        return Ok(());
    }

    let guild_id = match msg.guild_id() {
        Some(guild_id) => guild_id.0.to_string(),
        None => {
            println!("oh no! no guild id??");
            return Ok(());
        },
    };
    
    let data = ctx.data.lock();
    let ws_tx = data.get::<keys::LavalinkSocketSender>().unwrap().clone();

    let _ = msg.channel_id.say(&format!("changing volume to {}/150", volume));

    let _ = ws_tx.lock().unwrap().send(message::volume(&guild_id, volume));

    Ok(())
}