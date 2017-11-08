use std::str::FromStr;
use std::string::ToString;

#[derive(Debug)]
pub enum Opcode {
    // client -> server | Make the server queue a voice connection
    // guild_id: String, channel_id: String
    Connect,

    // client -> server | Provide an intercepted voice server update
    // session_id: String, event: String
    VoiceUpdate,

    // client -> server | Close a voice connection
    // guild_id: String
    Disconnect,

    // server -> client | Request to check if the VC or Guild exists, and that we have access to the VC
    ValidationReq,

    // client -> server | Response to ValidationRequest
    // guild_id: String, channel_id: Option<String>, valid: bool
    ValidationRes,

    // server- > client | Request to check if a shard's mainWS is connected
    IsConnectedReq,

    // client -> server | Response to IsConnectedRequest
    // shard_id: i32, connected: bool
    IsConnectedRes,

    // client -> server | Cause the player to play a track
    // guild_id: String, track: String, start_time: i64
    Play,

    // client -> server | Cause the player to stop
    // guild_id: String
    Stop,

    // client -> server | Set player pause
    // guild_id: String, pause: bool
    Pause,

    // client -> server | Make the player seek to a position of the track
    // guild_id: String, position: i64
    Seek,

    // client -> server | Set player volume from 1 to 150 (100 default)
    // guild_id: String, volume: i32
    Volume,

    // server -> client | Incoming message to forward to mainWS
    SendWS,

    // server -> client | Position information about a player
    PlayerUpdate,

    // server -> client | A collection of stats sent every minute
    Stats,

    // server -> client | Server emitted an event
    Event,

    // Unknown opcode
    Unknown,
}

impl ToString for Opcode {
    fn to_string(&self) -> String {
        // convert opcode's fmt::Debug name to lowerCamelCase
        format!("{:?}", *self).chars()
            .enumerate()
            .map(|(i, c)| {
                if c.is_uppercase() && i == 0 {
                    c.to_lowercase().next().unwrap()
                } else {
                    c
                }
            })
            .collect::<String>()
    }
}

impl FromStr for Opcode {
    type Err = Opcode;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::Opcode::*;

        let op = match s {
            "connect" => Connect,
            "voiceUpdate" => VoiceUpdate,
            "disconnect" => Disconnect,
            "validationReq" => ValidationReq,
            "validationRes" => ValidationRes,
            "isConnectedReq" => IsConnectedReq,
            "isConnectedRes" => IsConnectedRes,
            "play" => Play,
            "stop" => Stop,
            "pause" => Pause,
            "seek" => Seek,
            "volume" => Volume,
            "sendWS" => SendWS,
            "playerUpdate" => PlayerUpdate,
            "stats" => Stats,
            "event" => Event,
            _ => {
                return Err(Unknown);
            },
        };

        Ok(op)
    }
}