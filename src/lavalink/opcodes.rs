use std::str::FromStr;
use std::string::ToString;

#[derive(Debug)]
pub enum Opcode {
    // Make the server queue a voice connection
    // guild_id: String, channel_id: String
    Connect,

    // Provide an intercepted voice server update
    // session_id: String, event: String
    VoiceUpdate,

    // Close a voice connection
    // guild_id: String
    Disconnect,

    // Request to check if the VC or Guild exists, and that we have access to the VC
    ValidationReq,

    // Response to ValidationRequest
    // guild_id: String, channel_id: Option<String>, valid: bool
    // IMPLEMENTATION.md: channel_id is omitted if the request does not display the channel id
    // I think using Option<String> and not including when serializing is the best option here
    ValidationRes,

    // Request to check if a shard's mainWS is connected
    IsConnectedReq,

    // Response to IsConnectedRequest
    // shard_id: i32, connected: bool
    IsConnectedRes,

    // Cause the player to play a track
    // guild_id: String, track: String, start_time: ?
    // IMPLEMENTATION.md has start_time as "60000" so todo check the data type it is expecting
    Play,

    // Cause the player to stop
    // guild_id: String
    Stop,

    // Set player pause
    // guild_id: String, pause: bool
    Pause,

    // Make the player seek to a position of the track
    // guild_id: String, position: i64
    // here the position is not shown as a string in IMPLEMENTATION.md? todo check data types
    Seek,

    // Set player volume
    // guild_id: String, volume: i32
    // IMPLEMENTATION.md: Volume may range from 0 to 150. 100 is default.
    Volume,

    // Incoming message to forward to mainWS
    SendWS,

    // Position information about a player
    PlayerUpdate,

    // A collection of stats sent every minute
    Stats,

    // Server emitted an event
    Event,

    // Unknown opcode
    Unknown,
}

impl ToString for Opcode {
    fn to_string(&self) -> String {
        // convert opcode's fmt::Debug name to lowerCamelCase
        let mut buf = String::new();

        for (i, c) in format!("{:?}", *self).chars().enumerate() {
            if c.is_uppercase() && i == 0 {
                let _ = buf.push_str(c.to_lowercase().to_string().as_ref());
            } else {
                let _ = buf.push(c);
            }
        }

        buf
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