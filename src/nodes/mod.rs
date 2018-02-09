mod node;
mod node_manager;

pub use self::node::Node;
pub use self::node_manager::NodeManager;

use player::*;
use lavalink::stats::*;

use parking_lot::{Mutex, RwLock};
use serenity::client::bridge::gateway::ShardManager;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use websocket::OwnedMessage;

pub type NodeAudioPlayerManager = Arc<RwLock<AudioPlayerManager>>;
pub type NodeSender = Arc<Mutex<Sender<OwnedMessage>>>;
pub type NodeState = Arc<RwLock<State>>;
pub type SerenityShardManager = Arc<Mutex<ShardManager>>;

#[derive(Clone, Debug)]
pub struct NodeConfig {
    pub http_host: String,
    pub websocket_host: String,
    pub user_id: String,
    pub password: String,
    pub num_shards: u64,
}

#[derive(Clone, Debug, Default)]
pub struct State {
    pub stats: Option<RemoteStats>,
}

impl State {
    fn new() -> Self {
        Self::default()
    }
}
