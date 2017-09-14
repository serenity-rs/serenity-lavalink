use ::lavalink::opcodes::Opcode::VoiceUpdate;
use keys;

use std::sync::{Arc, Mutex};

use serenity::model::*;
use serenity::prelude::*;
use serenity::model::event::*;
use websocket::OwnedMessage;

pub struct GuildVoiceState {
    channel_id: Option<ChannelId>,
    endpoint: Option<String>,
    session_id: Option<String>,
    token: Option<String>,
}

impl GuildVoiceState {
    pub fn new() -> Self {
        Self {
            channel_id: None,
            endpoint: None,
            session_id: None,
            token: None,
        }
    }
}

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

    https://github.com/zeyla/serenity/blob/master/src/gateway/shard.rs#L712
    https://github.com/zeyla/serenity/blob/master/src/voice/handler.rs#L308
    */

    fn on_voice_state_update(&self, ctx: Context, guild_id: Option<GuildId>, voice_state: VoiceState) {
        let guild_id = match guild_id {
            Some(guild_id) => guild_id,
            None => {
                println!("got voice state without guild_id?");
                return
            }
        };

        println!("voice state update {:?} for guild {}", &voice_state, &guild_id);

        let data = ctx.data.lock();

        let guild_states = data.get::<keys::GuildVoiceState>().unwrap();
        let mut guild_states = guild_states.lock().unwrap();

        if !guild_states.contains_key(&guild_id) {
            let _ = guild_states.insert(guild_id.clone(), Arc::new(Mutex::new(GuildVoiceState::new())));
        }

        let guild_state = guild_states.get(&guild_id).unwrap().clone();
        let mut guild_state = guild_state.lock().unwrap();

        match voice_state.channel_id {
            Some(channel_id) => {
                guild_state.channel_id = Some(channel_id);
            },
            _ => {},
        }

        guild_state.session_id = Some(voice_state.session_id);

        match voice_state.token {
            Some(token) => {
                guild_state.token = Some(token);
            },
            _ => {},
        }
    }

    fn on_voice_server_update(&self, ctx: Context, event: VoiceServerUpdateEvent) {
        let guild_id = event.guild_id.unwrap();
        let guild_id_str = &guild_id.0.to_string();

        println!("voice server update {:?}", event);

        let data = ctx.data.lock();

        let guild_states = data.get::<keys::GuildVoiceState>().unwrap();
        let guild_states = guild_states.lock().unwrap();

        if !guild_states.contains_key(&guild_id) {
            // guild states doesn't contain the guild so no voice state update received yet, lets
            // just return for now
            println!("no guild state exists for guild {}", &guild_id);
            return;
        }

        let guild_state = guild_states.get(&guild_id).unwrap().clone();
        let mut guild_state = guild_state.lock().unwrap();

        match event.endpoint {
            Some(endpoint) => {
                guild_state.endpoint = Some(endpoint);
            },
            _ => {},
        }

        guild_state.token = Some(event.token);

        let ws_tx = data.get::<keys::LavalinkSocketSender>().unwrap().clone();

        let data = json!({
            "op": VoiceUpdate.to_string(),
            "sessionId": guild_state.session_id.clone().unwrap(),
            "guildId": guild_id_str,
            "event": {
                "token": guild_state.token.clone().unwrap(),
                "guild_id": guild_id_str,
                "endpoint": guild_state.endpoint.clone().unwrap(),
            }
        }).to_string();

        let _ = ws_tx.lock().unwrap().send(OwnedMessage::Text(data));
    }
}