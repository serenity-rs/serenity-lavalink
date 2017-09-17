use handler;
use lavalink;
use serenity;
use std::{collections, sync};
use typemap::Key;
use websocket;

pub struct GuildVoiceState;

impl Key for GuildVoiceState {
    type Value = sync::Mutex<collections::HashMap<serenity::model::GuildId, sync::Arc<sync::Mutex<handler::GuildVoiceState>>>>;
}

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