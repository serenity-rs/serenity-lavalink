use serenity::model::*;
use serenity::client::Context;
use serenity::framework::standard::Args;

pub fn ping(_: &mut Context, msg: &Message, _: Args) -> Result<(), String> {
    let _ = msg.channel_id.say("Pong!");

    Ok(())
}