use lavalink;
use serenity;
use std::sync;
use typemap::Key;
use websocket;

pub struct LavalinkAudioPlayerManager;

impl Key for LavalinkAudioPlayerManager {
    type Value = sync::Arc<sync::Mutex<lavalink::player::AudioPlayerManager>>;
}

pub struct LavalinkConfig;

impl Key for LavalinkConfig {
    type Value = lavalink::config::Config;
}

pub struct LavalinkSocketSender;

impl Key for LavalinkSocketSender {
    type Value = sync::Arc<sync::Mutex<sync::mpsc::Sender<websocket::OwnedMessage>>>;
}

pub struct LavalinkSocketState;

impl Key for LavalinkSocketState {
    type Value = sync::Arc<sync::Mutex<lavalink::socket::SocketState>>;
}

pub struct SerenityCloseHandle;

impl Key for SerenityCloseHandle {
    type Value = serenity::client::CloseHandle;
}