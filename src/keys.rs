use lavalink;
use serenity;
use std::sync;
use typemap::Key;
use websocket;

pub struct LavalinkConfig;

impl Key for LavalinkConfig {
    type Value = lavalink::config::Config;
}

pub struct LavalinkSocketSender;

impl Key for LavalinkSocketSender {
    type Value = sync::Arc<sync::Mutex<sync::mpsc::Sender<websocket::OwnedMessage>>>;
}

pub struct SerenityCloseHandle;

impl Key for SerenityCloseHandle {
    type Value = serenity::client::CloseHandle;
}

pub struct CurrentUserId;

impl Key for CurrentUserId {
    type Value = serenity::model::UserId;
}