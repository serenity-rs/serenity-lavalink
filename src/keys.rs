use lavalink;
use serenity;
use std;
use typemap::Key;
use websocket;

pub struct LavalinkConfig;

impl Key for LavalinkConfig {
    type Value = lavalink::config::Config;
}

pub struct LavalinkSocketSender;

impl Key for LavalinkSocketSender {
    type Value = std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Sender<websocket::OwnedMessage>>>;
}

pub struct SerenityCloseHandle;

impl Key for SerenityCloseHandle {
    type Value = serenity::client::CloseHandle;
}