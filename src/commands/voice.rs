use serenity::model::*;
use serenity::client::Context;
use serenity::framework::standard::Args;

const GUILD_ID: GuildId = GuildId(272410239947767808); // dabBot guild
const VOICE_CHANNEL_ID: ChannelId = ChannelId(320643590986399749); // TESTING!!! voice channel

pub fn join(ctx: &mut Context, msg: &Message, _: Args) -> Result<(), String> {
    let mut shard = ctx.shard.lock();
    let manager = &mut shard.manager;

    let handler = manager.join(GUILD_ID, VOICE_CHANNEL_ID);

    let _ = msg.channel_id.say(&format!("{:?}", VOICE_CHANNEL_ID));

    Ok(())
}

pub fn leave(ctx: &mut Context, msg: &Message, _: Args) -> Result<(), String> {
    let mut shard = ctx.shard.lock();

    if shard.manager.get(GUILD_ID).is_some() {
        shard.manager.remove(GUILD_ID);

        let _ = msg.channel_id.say(&format!("{:?}", VOICE_CHANNEL_ID));
    } else {
        let _ = msg.channel_id.say(&format!("not in {:?}", VOICE_CHANNEL_ID));
    }

    Ok(())
}