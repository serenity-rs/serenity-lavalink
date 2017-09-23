use keys;

use serenity::client::Context;
use serenity::framework::standard::Args;
use serenity::model::*;

fn toggle_paused(ctx: &mut Context, msg: &Message, pause: bool) -> Result<(), String> {
    let guild_id = match msg.guild_id() {
        Some(guild_id) => guild_id.0,
        None => {
            println!("oh no! no guild id??");
            return Ok(());
        },
    };

    {
        let data = ctx.data.lock();

        let player_manager = data.get::<keys::LavalinkAudioPlayerManager>()
            .expect("keys::LavalinkAudioPlayerManager not present in Context::data");

        let player_manager = player_manager.lock()
            .expect("could not obtain lock on player manager");

        let player = match player_manager.get_player(&guild_id) {
            Some(player) => player,
            None => {
                let _ = msg.channel_id.say("this guild does not have an audio player");
                return Ok(());
            }
        };

        player.lock().as_mut().map(|lock| lock.pause(pause))
            .expect("error obtaining lock on player and pausing");
    }

    let _ = msg.channel_id.say(
        String::from(if pause { "paus" } else { "resum" }) + "ed music"
    );
    
    Ok(())
}

pub fn pause(ctx: &mut Context, msg: &Message, _: Args) -> Result<(), String> {
    toggle_paused(ctx, msg, true)
}

pub fn resume(ctx: &mut Context, msg: &Message, _: Args) -> Result<(), String> {
    toggle_paused(ctx, msg, false)
}