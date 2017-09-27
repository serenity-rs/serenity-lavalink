use keys;

command!(shutdown(ctx, msg) {
    let _ = msg.channel_id.say("Shutting down!");

    let data = ctx.data.lock();

    let close_handle = data.get::<keys::SerenityCloseHandle>()
        .expect("keys::SerenityCloseHandle not present in Context::data");

    close_handle.close();
});