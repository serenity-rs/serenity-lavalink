use std::sync::{Arc, RwLock};
use lavalink::node::{NodeConfig, NodeManager};
use serenity::client::CloseHandle;
use typemap::Key;

pub struct LavalinkConfig;

impl Key for LavalinkConfig {
    type Value = NodeConfig;
}

pub struct LavalinkNodeManager;

impl Key for LavalinkNodeManager {
    type Value = Arc<RwLock<NodeManager>>;
}

pub struct SerenityCloseHandle;

impl Key for SerenityCloseHandle {
    type Value = CloseHandle;
}