use super::opcodes::Opcode::*;

use serde_json::Value;
use websocket::OwnedMessage;

#[inline]
fn from_json(json: Value) -> OwnedMessage {
    OwnedMessage::Text(json.to_string())
}

pub fn connect(guild_id: &str, channel_id: &str) -> OwnedMessage {
    from_json(json!({
        "op": Connect.to_string(),
        "guildId": guild_id,
        "channelId": channel_id,
    }))
}

pub fn voice_update(session_id: &str, guild_id: &str, token: &str, endpoint: &str) -> OwnedMessage {
    from_json(json!({
        "op": VoiceUpdate.to_string(),
        "sessionId": session_id,
        "guildId": guild_id,
        "event": {
            "token": token,
            "guild_id": guild_id,
            "endpoint": endpoint,
        },
    }))
}

pub fn disconnect(guild_id: &str) -> OwnedMessage {
    from_json(json!({
        "op": Connect.to_string(),
        "guildId": guild_id,
    }))
}

pub fn validation_response(guild_id: &str, channel_id: Option<&str>, valid: bool) -> OwnedMessage {
    let json = match channel_id {
        Some(channel_id) => {
            json!({
                "op": ValidationResponse.to_string(),
                "guildId": guild_id,
                "channelId": channel_id,
                "valid": valid,
            })
        },
        None => {
            json!({
                "op": ValidationResponse.to_string(),
                "guildId": guild_id,
                "valid": valid,
            })
        },
    };
    
    from_json(json)
}

pub fn is_connected_response(shard_id: u64, connected: bool) -> OwnedMessage {
    from_json(json!({
        "op": IsConnectedResponse.to_string(),
        "shardId": shard_id,
        "connected": connected,
    }))
}

pub fn play(guild_id: &str, track: &str) -> OwnedMessage {
    from_json(json!({
        "op": Play.to_string(),
        "guildId": guild_id,
        "track": track,
    }))
}

pub fn stop(guild_id: &str) -> OwnedMessage {
    from_json(json!({
        "op": Stop.to_string(),
        "guildId": guild_id,
    }))
}

pub fn pause(guild_id: &str, pause: bool) -> OwnedMessage {
    from_json(json!({
        "op": Pause.to_string(),
        "guildId": guild_id,
        "pause": pause,
    }))
}

pub fn seek(guild_id: &str, position: i64) -> OwnedMessage {
    from_json(json!({
        "op": Seek.to_string(),
        "guildId": guild_id,
        "position": position,
    }))
}

pub fn volume(guild_id: &str, volume: i32) -> OwnedMessage {
    from_json(json!({
        "op": Volume.to_string(),
        "guildId": guild_id,
        "volume": volume,
    }))
}