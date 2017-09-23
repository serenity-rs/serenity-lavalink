use keys;

use serenity::model::*;
use serenity::client::Context;
use serenity::framework::standard::Args;

pub fn shutdown(ctx: &mut Context, msg: &Message, _: Args) -> Result<(), String> {
    let _ = msg.channel_id.say("Shutting down!");

    let data = ctx.data.lock();

    let close_handle = data.get::<keys::SerenityCloseHandle>()
        .expect("keys::SerenityCloseHandle not present in Context::data");

    close_handle.close();

    Ok(())
}