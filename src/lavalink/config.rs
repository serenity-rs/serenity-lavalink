#[derive(Clone)]
pub struct Config {
    pub http_host: String,
    pub websocket_host: String,
    pub user_id: String,
    pub password: String,
    pub num_shards: u64,
}