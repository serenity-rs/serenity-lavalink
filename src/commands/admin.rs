use keys::SerenityCloseHandle;

use serenity::model::*;
use serenity::client::{Context, CloseHandle};
use serenity::framework::standard::Args;

pub fn stop(ctx: &mut Context, msg: &Message, _: Args) -> Result<(), String> {
    let _ = msg.channel_id.say("Shutting down!");

    let data = &*ctx.data.lock();
    let close_handle = data.get::<SerenityCloseHandle>().unwrap();

    close_handle.close();

    Ok(())
}