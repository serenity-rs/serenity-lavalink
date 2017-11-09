#[derive(Clone, Debug, Default, Deserialize)]
pub struct FrameStats {
    // average frames sent per minute
    pub sent: i32,
    // average frames nulled per minute
    pub nulled: i32,
    // average frames deficit per minute
    pub deficit: i32,
}

#[derive(Clone, Debug, Default, Deserialize)]
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
