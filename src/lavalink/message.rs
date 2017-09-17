use super::opcodes::Opcode::*;

use serde_json::Value;
use websocket::OwnedMessage;

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

pub fn disconnect(guild_id: &str) -> OwnedMessage {
    from_json(json!({
        "op": Connect.to_string(),
        "guildId": guild_id,
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