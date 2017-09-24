use lavalink::config::Config;
use lavalink::socket::{SocketAudioPlayerManager, SocketSender, SocketState};
use serenity::client::CloseHandle;
use typemap::Key;

pub struct LavalinkAudioPlayerManager;

impl Key for LavalinkAudioPlayerManager {
    type Value = SocketAudioPlayerManager;
}

pub struct LavalinkConfig;

impl Key for LavalinkConfig {
    type Value = Config;
}

pub struct LavalinkSocketSender;

impl Key for LavalinkSocketSender {
    type Value = SocketSender;
}

pub struct LavalinkSocketState;

impl Key for LavalinkSocketState {
    type Value = SocketState;
}

pub struct SerenityCloseHandle;

impl Key for SerenityCloseHandle {
    type Value = CloseHandle;
}