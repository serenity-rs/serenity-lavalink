use typemap::Key;
use serenity::client::CloseHandle;

pub struct CloseHandleKey;

impl Key for CloseHandleKey {
    type Value = CloseHandle;
}

command!(stop(ctx, msg) {
    let _ = msg.channel_id.say("Shutting down serenity!");

    let data = &*ctx.data.lock();
    let close_handle = data.get::<CloseHandleKey>().unwrap();

    close_handle.close();
});