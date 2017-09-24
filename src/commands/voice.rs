use keys;

use lavalink::message;
use serenity::client::Context;
use serenity::framework::standard::Args;
use serenity::model::*;

pub fn join(ctx: &mut Context, msg: &Message, _: Args) -> Result<(), String> {
    let guild = match msg.guild() {
        Some(guild) => guild,
        None => {
            println!("None of them guilds :((");
            return Ok(());
        },
    };
    
    let guild = guild.read().unwrap();
    let user_id = msg.author.id;
    
    let voice_state = match guild.voice_states.get(&user_id) {
        Some(voice_state) => voice_state,
        None => {
            let _ = msg.channel_id.say("You must be in a voice channel....");
            return Ok(());
        },
    };

    let _ = msg.channel_id.say(format!("Joining voice channel <#{}> ({})", &voice_state.channel_id.unwrap().0, &voice_state.channel_id.unwrap().0));
    
    let data = ctx.data.lock();
    let ws_tx = data.get::<keys::LavalinkSocketSender>().unwrap().clone();

    let _ = ws_tx.lock().unwrap().send(message::connect(&guild.id.0.to_string(), &voice_state.channel_id.unwrap().to_string()));

    Ok(())
}

pub fn leave(ctx: &mut Context, msg: &Message, _: Args) -> Result<(), String> {
    let guild_id = match msg.guild_id() {
        Some(guild_id) => guild_id.0.to_string(),
        None => {
            println!("oh no! no guild id??");
            return Ok(());
        },
    };

    let _ = msg.channel_id.say("leaving channel");
    
    let data = ctx.data.lock();
    let ws_tx = data.get::<keys::LavalinkSocketSender>().unwrap().clone();

    let _ = ws_tx.lock().unwrap().send(message::disconnect(&guild_id));

    Ok(())
}