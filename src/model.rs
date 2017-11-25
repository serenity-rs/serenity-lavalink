use serde_json;
use super::opcodes::Opcode;
use websocket::OwnedMessage;
use ::prelude::*;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Connect {
    pub channel_id: String,
    pub guild_id: String,
    op: Opcode,
}

impl Connect {
    pub fn new<S: Into<String>>(channel_id: S, guild_id: S) -> Self {
        Self {
            channel_id: channel_id.into(),
            guild_id: guild_id.into(),
            op: Opcode::Connect,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Disconnect {
    guild_id: String,
    op: Opcode,
}

impl Disconnect {
    pub fn new<S: Into<String>>(guild_id: S) -> Self {
        Self {
            guild_id: guild_id.into(),
            op: Opcode::Disconnect,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IsConnectedResponse {
    pub connected: bool,
    op: Opcode,
    pub shard_id: u64,
}

impl IsConnectedResponse {
    pub fn new(shard_id: u64, connected: bool) -> Self {
        Self {
            op: Opcode::IsConnectedRes,
            connected,
            shard_id,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pause {
    pub guild_id: String,
    op: Opcode,
    pub pause: bool,
}

impl Pause {
    pub fn new<S: Into<String>>(guild_id: S, pause: bool) -> Self {
        Self {
            guild_id: guild_id.into(),
            op: Opcode::Pause,
            pause,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Play {
    pub end_time: Option<u64>,
    pub guild_id: String,
    op: Opcode,
    pub start_time: Option<u64>,
    pub track: String,
}

impl Play {
    pub fn new<S: Into<String>>(
        guild_id: S,
        track: S,
        start_time: Option<u64>,
        end_time: Option<u64>,
    ) -> Self {
        Self {
            guild_id: guild_id.into(),
            op: Opcode::Play,
            track: track.into(),
            end_time,
            start_time,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Seek {
    pub guild_id: String,
    op: Opcode,
    pub seek: bool,
}

impl Seek {
    pub fn new<S: Into<String>>(guild_id: S, seek: bool) -> Self {
        Self {
            guild_id: guild_id.into(),
            op: Opcode::Seek,
            seek,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stop {
    pub guild_id: String,
    op: Opcode,
}

impl Stop {
    pub fn new<S: Into<String>>(guild_id: S) -> Self {
        Self {
            guild_id: guild_id.into(),
            op: Opcode::Stop,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResponse {
    pub channel_id: Option<String>,
    pub guild_id: String,
    op: Opcode,
    pub valid: bool,
}

impl ValidationResponse {
    pub fn new<S>(guild_id: S, channel_id: Option<S>, valid: bool) -> Self
        where S: Into<String> {
        Self {
            channel_id: channel_id.map(Into::into),
            guild_id: guild_id.into(),
            op: Opcode::ValidationRes,
            valid,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceUpdate {
    pub endpoint: String,
    pub event: VoiceUpdateEvent,
    pub guild_id: String,
    op: Opcode,
    pub session_id: String,
    pub token: String,
}

impl VoiceUpdate {
    pub fn new<S>(session_id: S, guild_id: S, token: S, endpoint: S) -> Self
        where S: Into<String> {
        let endpoint = endpoint.into();
        let guild_id = guild_id.into();
        let token = token.into();

        Self {
            endpoint: endpoint.clone(),
            event: VoiceUpdateEvent {
                endpoint: endpoint,
                guild_id: guild_id.clone(),
                token: token.clone(),
            },
            op: Opcode::VoiceUpdate,
            session_id: session_id.into(),
            guild_id,
            token,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceUpdateEvent {
    pub endpoint: String,
    pub guild_id: String,
    pub token: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
    pub guild_id: String,
    op: Opcode,
    pub volume: i32,
}

impl Volume {
    pub fn new<S: Into<String>>(guild_id: S, volume: i32) -> Self {
        Self {
            guild_id: guild_id.into(),
            op: Opcode::Volume,
            volume,
        }
    }
}

pub trait IntoWebSocketMessage {
    fn into_ws_message(self) -> Result<OwnedMessage>;
}

macro_rules! impl_stuff_for_model {
    ($($model: ident),*) => {
        $(
            /// Implementation for retrieving the opcode of a model.
            impl $model {
                /// Retrieves the opcode for the model.
                pub fn opcode(&self) -> Opcode {
                    self.op.clone()
                }
            }

            impl IntoWebSocketMessage for $model {
                /// Serializes the model into a JSON string, wrapping it in an
                /// owned WebSocket message.
                fn into_ws_message(self) -> Result<OwnedMessage> {
                    serde_json::to_string(&self)
                        .map(OwnedMessage::Text)
                        .map_err(From::from)
                }
            }
        )*
    };
}

impl_stuff_for_model! {
    Connect,
    Disconnect,
    IsConnectedResponse,
    Pause,
    Play,
    Seek,
    Stop,
    ValidationResponse,
    VoiceUpdate,
    Volume
}
