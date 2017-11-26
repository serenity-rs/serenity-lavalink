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
pub struct MemoryStats {
    pub free: i64,
    pub used: i64,
    pub allocated: i64,
    pub reservable: i64,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuStats {
    pub cores: i32,
    pub system_load: f64,
    pub lavalink_load: f64,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteStats {
    pub players: i32,
    pub playing_players: i32,
    pub uptime: i64,
    pub memory: MemoryStats,
    pub cpu: CpuStats,
    pub frame_stats: Option<FrameStats>,
}
