use keys;

use lavalink::message;

use serenity::client::CACHE;
use serenity::model::*;
use serenity::prelude::*;
use serenity::model::event::*;

pub struct Handler;

impl EventHandler for Handler {
    fn on_ready(&self, _: Context, ready: Ready) {
        println!("READY event for {}", ready.user.name);
    }

    fn on_voice_server_update(&self, ctx: Context, event: VoiceServerUpdateEvent) {
        let guild_id = match event.guild_id {
            Some(guild_id) => guild_id,
            None => {
                println!("got voice server update without a guild id?");
                return;
            }
        };

        let cache = CACHE.read().unwrap();

        let guild = match cache.guilds.get(&guild_id) {
            Some(guild) => guild.read().unwrap(),
            None => {
                println!("guild from voice server update not in cache?");
                return;
            },
        };

        let current_user_id = cache.user.id;

        let voice_state = match guild.voice_states.get(&current_user_id) {
            Some(voice_state) => voice_state,
            None => {
                println!("no voice state found for user {:?}", &current_user_id);
                return;
            }
        };

        let guild_id = guild_id.to_string();

        let endpoint = match event.endpoint {
            Some(endpoint) => endpoint,
            None => {
                println!("no endpoint found in voice server update!");
                return;
            }
        };

        let json_data = message::voice_update(&voice_state.session_id, &guild_id, &event.token, &endpoint);

        let data = ctx.data.lock();
        let ws_tx = data.get::<keys::LavalinkSocketSender>().unwrap().clone();
        let _ = ws_tx.lock().unwrap().send(json_data);
    }
}