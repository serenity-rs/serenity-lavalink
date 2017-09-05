use serenity::voice;
use serenity::model::*;
use serenity::prelude::*;
use serenity::model::event::*;

pub struct Handler;

impl EventHandler for Handler {
    fn on_ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    fn on_message(&self, _: Context, message: Message) {
        println!("got msg: {}", message.content);
    }

    fn on_voice_server_update(&self, _: Context, event: VoiceServerUpdateEvent) {
        // todo work out if i need to stop serenity doing internal shit
        // https://github.com/zeyla/serenity/blob/master/src/gateway/shard.rs#L712
        // https://github.com/zeyla/serenity/blob/master/src/voice/handler.rs#L308


        let guild_id = event.guild_id.unwrap().0;

        println!("guild_id: {}", guild_id);
    }
}