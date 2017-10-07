use keys;

use lavalink::message;
use serenity::client::CACHE;
use serenity::model::*;
use serenity::model::event::*;
use serenity::prelude::*;

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
            },
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
            },
        };

        let guild_id_u64 = guild_id.0;
        let guild_id_str = guild_id.to_string();

        let endpoint = match event.endpoint {
            Some(endpoint) => endpoint,
            None => {
                println!("no endpoint found in voice server update!");
                return;
            },
        };

        let data = ctx.data.lock();
        //let ws_tx = data.get::<keys::LavalinkSocketSender>().unwrap().clone();
        //let _ = ws_tx.lock().unwrap().send(json_data);

        let node_manager = data.get::<keys::LavalinkNodeManager>()
            .expect("could not get key::LavalinkNodeManager from Context::data")
            .read()
            .expect("could not get read lock on node_manager");

        /*let nodes = node_manager.nodes.read().expect("could not get read lock on nodes");

        for node in nodes.iter() {
            let node = node.clone();
            let sender = node.sender.clone();

            let sender = sender.lock().expect("could not get lock on node sender");

            let json_data = message::voice_update(&voice_state.session_id, &guild_id, &event.token, &endpoint);

            let _ = sender.send(json_data)
                .map_err(|e| panic!("error sending json data: {:?}", e));
        }*/

        let player_manager = node_manager.player_manager.read()
            .expect("could not get write lock on player manager");

        let player = match player_manager.get_player(&guild_id_u64) {
            Some(player) => player,
            None => {
                panic!("got voice server update for guild {} without player", &guild_id);
            }
        };

        let reader = player.lock().expect("could not get lock on player");
        let sender = reader.sender.lock().expect("could not get lock on node sender");
        let json_data = message::voice_update(&voice_state.session_id, &guild_id_str, &event.token, &endpoint);

        let _ = sender.send(json_data)
            .map_err(|e| panic!("error sending json data: {:?}", e));
    }
}