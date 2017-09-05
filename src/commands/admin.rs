use serenity::model::*;
use serenity::client::{Context, CloseHandle};
use serenity::framework::standard::Args;
use typemap::Key;

pub struct CloseHandleKey;

impl Key for CloseHandleKey {
    type Value = CloseHandle;
}

pub fn stop(ctx: &mut Context, msg: &Message, _: Args) -> Result<(), String> {
    let _ = msg.channel_id.say("Shutting down!");

    let data = &*ctx.data.lock();
    let close_handle = data.get::<CloseHandleKey>().unwrap();

    close_handle.close();

    Ok(())
}