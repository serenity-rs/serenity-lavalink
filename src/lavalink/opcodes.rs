use std::string::ToString;
use std::str::FromStr;

#[derive(Debug)]
pub enum Opcode {
    // Make the server queue a voice connection
    Connect,

    // Provide an intercepted voice server update
    VoiceUpdate,

    // Close a voice connection
    Disconnect,

    // Request to check if the VC or Guild exists, and that we have access to the VC
    ValidationRequest,

    // Response to ValidationRequest
    ValidationResponse,

    // Request to check if a shard's mainWS is connected
    IsConnectedRequest,

    // Response to IsConnectedRequest
    IsConnectedResponse,

    // Cause the player to play a track
    Play,

    // Cause the player to stop
    Stop,

    // Set player pause
    Pause,

    // Make the player seek to a position of the track
    Seek,

    // Set player volume
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
        use self::Opcode::*;

        match *self {
            Connect => "connect",
            VoiceUpdate => "voiceUpdate",
            Disconnect => "disconnect",
            ValidationRequest => "validationReq",
            ValidationResponse => "validationRes",
            IsConnectedRequest => "isConnectedReq",
            IsConnectedResponse => "isConnectedRes",
            Play => "play",
            Stop => "stop",
            Pause => "pause",
            Seek => "seek",
            Volume => "volume",
            SendWS => "sendWS",
            PlayerUpdate => "playerUpdate",
            Stats => "stats",
            Event => "event",
            Unknown => "unknown",
        }.to_string()
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
            "validationReq" => ValidationRequest,
            "validationRes" => ValidationResponse,
            "isConnectedReq" => IsConnectedRequest,
            "isConnectedRes" => IsConnectedResponse,
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