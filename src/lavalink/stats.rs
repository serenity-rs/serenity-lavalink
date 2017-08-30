use serde_json::Value;

#[derive(Debug)]
pub struct FrameStats {
    // average frames sent per minute
    sent: i32,
    // average frames nulled per minute
    nulled: i32,
    // average frames deficit per minute
    deficit: i32,
}

#[derive(Debug)]
pub struct RemoteStats {
    pub players: i32,
    pub playing_players: i32,
    pub uptime: i64,

    pub mem_free: i64,
    pub mem_used: i64,
    pub mem_allocated: i64,
    pub mem_reservable: i64,

    pub cpu_cores: i32,
    pub system_load: f64,
    pub lavalink_load: f64,

    pub frame_stats: Option<FrameStats>,
}

impl RemoteStats {
    pub fn from_json(json: &Value) -> Self {
        let memory = json["memory"].as_object().unwrap();
        let cpu = json["cpu"].as_object().unwrap();

        let mut stats = Self {
            players: json["players"].as_i64().unwrap() as i32,
            playing_players: json["playingPlayers"].as_i64().unwrap() as i32,
            uptime: json["uptime"].as_i64().unwrap(),

            mem_free: memory["free"].as_i64().unwrap(),
            mem_used: memory["used"].as_i64().unwrap(),
            mem_allocated: memory["allocated"].as_i64().unwrap(),
            mem_reservable: memory["reservable"].as_i64().unwrap(),

            cpu_cores: cpu["cores"].as_i64().unwrap() as i32,
            system_load: cpu["systemLoad"].as_f64().unwrap(),
            lavalink_load: cpu["lavalinkLoad"].as_f64().unwrap(),

            frame_stats: None,
        };

        if let Some(frames) = json.get("frameStats") {
            let frames = frames.as_object().unwrap();

            let frame_stats = FrameStats {
                sent: frames["sent"].as_i64().unwrap() as i32,
                nulled: frames["nulled"].as_i64().unwrap() as i32,
                deficit: frames["deficit"].as_i64().unwrap() as i32,
            };

            stats.frame_stats = Some(frame_stats);
        }

        stats
    }
}