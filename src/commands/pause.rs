use keys;

use lavalink::message;

use serenity::model::*;
use serenity::client::Context;
use serenity::framework::standard::Args;

fn toggle_paused(ctx: &mut Context, msg: &Message, pause: bool) -> Result<(), String> {
    let guild_id = match msg.guild_id() {
        Some(guild_id) => guild_id.0.to_string(),
        None => {
            println!("oh no! no guild id??");
            return Ok(());
        },
    };

    {
        let data = ctx.data.lock();
        let ws_tx = data.get::<keys::LavalinkSocketSender>().unwrap();

        let _ = ws_tx.lock().unwrap().send(message::pause(&guild_id, pause));
    }

    let _ = msg.channel_id.say(
        String::from(if pause { "paused" } else { "resuming" }) + " music"
    );
    
    Ok(())
}

pub fn pause(ctx: &mut Context, msg: &Message, _: Args) -> Result<(), String> {
    toggle_paused(ctx, msg, true)
}

pub fn resume(ctx: &mut Context, msg: &Message, _: Args) -> Result<(), String> {
    toggle_paused(ctx, msg, false)
}