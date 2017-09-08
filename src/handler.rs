extern crate serde_json;

use ::lavalink::opcodes::Opcode::VoiceUpdate;
use keys::LavalinkSocketSender;

use serenity::voice;
use serenity::model::*;
use serenity::prelude::*;
use serenity::model::event::*;
use websocket::OwnedMessage;

const GUILD_ID: GuildId = GuildId(272410239947767808); // dabBot guild
const VOICE_CHANNEL_ID: ChannelId = ChannelId(320643590986399749); // TESTING!!! voice channel

pub struct Handler;

impl EventHandler for Handler {
    fn on_ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    fn on_message(&self, _: Context, message: Message) {
        println!("got msg: {}", message.content);
    }

    /*
    If our request succeeded, the gateway will respond with two events—a Voice State Update event
    and a Voice Server Update event—meaning your library must properly wait for both events before
    continuing. The first will contain a new key, session_id, and the second will provide voice
    server information we can use to establish a new voice connection
    */

    fn on_voice_state_update(&self, _: Context, _: Option<GuildId>, _: VoiceState) {
        unimplemented!()
    }

    fn on_voice_server_update(&self, ctx: Context, event: VoiceServerUpdateEvent) {
        // todo work out if i need to stop serenity doing internal shit
        // https://github.com/zeyla/serenity/blob/master/src/gateway/shard.rs#L712
        // https://github.com/zeyla/serenity/blob/master/src/voice/handler.rs#L308

        let guild_id = event.guild_id.unwrap();
        let guild_id_str = &guild_id.0.to_string();

        let mut shard = ctx.shard.lock();
        let manager = &mut shard.manager;

        let handler_exists = manager.get(guild_id).is_some();

        let handler = if handler_exists {
            manager.get(guild_id).unwrap()
        } else {
            manager.join(guild_id, VOICE_CHANNEL_ID)
        };

        let data = &*ctx.data.lock();
        let ws_tx = data.get::<LavalinkSocketSender>().unwrap().clone();

        println!("session_id: {:?}", handler.session_id.clone());

        /*let data = json!({
            "op": VoiceUpdate.to_string(),
            "sessionId": handler.session_id.clone().unwrap(),
            "guildId": guild_id_str,
            "event": {
                "token": handler.token,
                "guild_id": guild_id_str,
                "endpoint": handler.endpoint,
            }
        }).to_string();

        println!("voice state update = {}", &data);

        let _ = ws_tx.lock().unwrap().send(OwnedMessage::Text(data));*/
    }
}