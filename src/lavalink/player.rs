use std::collections::HashMap;
use std::marker::Send;
use std::sync::Arc;

use serenity::model::GuildId;

pub struct AudioPlayer<T: AudioPlayerListener> {
    pub listener: Option<Box<T>>,
    pub track: Option<String>,
    pub time: i64,
    pub position: i64,
    pub paused: bool,
}

impl<T: AudioPlayerListener> AudioPlayer<T> {
    pub fn new() -> Self {
        Self {
            listener: None,
            track: None,
            time: 0,
            position: 0,
            paused: false,
        }
    }
}

pub trait AudioPlayerListener: Send {
    fn on_player_pause(player: &AudioPlayer<Self>);

    fn on_player_resume(player: &AudioPlayer<Self>);

    fn on_track_start(player: &AudioPlayer<Self>, track: String);

    fn on_track_end(player: &AudioPlayer<Self>, track: String, reason: String);

    fn on_track_exception(player: &AudioPlayer<Self>, track: String, exception: String);

    fn on_track_stuck(player: &AudioPlayer<Self>, track: String, threshold_ms: i64);
}