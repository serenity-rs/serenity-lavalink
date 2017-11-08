use super::message;
use super::node::NodeSender;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use websocket::OwnedMessage;
use ::prelude::*;

type PlayerPauseHandler = fn(&AudioPlayer);
type PlayerResumeHandler = fn(&AudioPlayer);
type TrackStartHandler = fn(&AudioPlayer, &str);
type TrackEndHandler = fn(&AudioPlayer, &str, &str);
type TrackExceptionHandler = fn(&AudioPlayer, &str, &str);
type TrackStuckHandler = fn(&AudioPlayer, &str, i64);

pub struct AudioPlayerListener {
    pub on_player_pause: PlayerPauseHandler,
    pub on_player_resume: PlayerResumeHandler,
    pub on_track_start: TrackStartHandler,
    pub on_track_end: TrackEndHandler,
    pub on_track_exception: TrackExceptionHandler,
    pub on_track_stuck: TrackStuckHandler,
}

impl AudioPlayerListener {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_player_pause(mut self, handler: PlayerPauseHandler) -> Self {
        self.on_player_pause = handler;
        self
    }

    pub fn with_player_resume(mut self, handler: PlayerResumeHandler) -> Self {
        self.on_player_resume = handler;
        self
    }

    pub fn with_track_start(mut self, handler: TrackStartHandler) -> Self {
        self.on_track_start = handler;
        self
    }

    pub fn with_track_end(mut self, handler: TrackEndHandler) -> Self {
        self.on_track_end = handler;
        self
    }

    pub fn with_track_exception(mut self, handler: TrackExceptionHandler) -> Self {
        self.on_track_exception = handler;
        self
    }

    pub fn with_track_stuck(mut self, handler: TrackStuckHandler) -> Self {
        self.on_track_stuck = handler;
        self
    }
}

impl Default for AudioPlayerListener {
    fn default() -> Self {
        Self {
            on_player_pause: |_| {},
            on_player_resume: |_| {},
            on_track_start: |_, _| {},
            on_track_end: |_, _, _| {},
            on_track_exception: |_, _, _| {},
            on_track_stuck: |_, _, _| {},
        }
    }
}

// todo potentially split state into child struct to avoid mutable reference of AudioPlayer
// where mutablity should not be nessesary for non state fields
pub struct AudioPlayer {
    pub sender: NodeSender,
    pub guild_id: u64,
    pub track: Option<String>,
    pub time: i64,
    pub position: i64,
    pub paused: bool,
    pub volume: i32,
    pub listeners: Vec<AudioPlayerListener>,
}

impl AudioPlayer {
    fn new(sender: NodeSender, guild_id: u64) -> Self {
        Self {
            sender,
            guild_id,
            track: None,
            time: 0,
            position: 0,
            paused: false,
            volume: 100,
            listeners: Vec::new(),
        }
    }

    #[inline]
    pub fn add_listener(&mut self, listener: AudioPlayerListener) {
        self.listeners.push(listener);
    }

    #[inline]
    fn send(&self, message: OwnedMessage) -> Result<()> {
        self.sender.lock().unwrap().send(message).map_err(From::from)
    }

    pub fn play(&mut self, track: &str) {
        let result = self.send(message::play(
            &self.guild_id.to_string(),
            track
        ));

        match result {
            Ok(_) => {
                self.track = Some(track.to_string());

                for listener in &self.listeners {
                    let on_track_start = &listener.on_track_start;
                    on_track_start(self, track);
                }
            },
            Err(e) => {
                println!("play websocket send error {:?}", e);
            },
        }
    }

    pub fn stop(&mut self) {
        let result = self.send(message::stop(
            &self.guild_id.to_string()
        ));

        match result {
            Ok(_) => {
                let track = self.track.clone().unwrap_or_else(|| "no track in state".to_string());
                self.track = None;

                for listener in &self.listeners {
                    let on_track_end = &listener.on_track_end;
                    on_track_end(self, &track, "no reason :) :dabs:");
                }

                println!("stopped playing track {:?}", track);
            },
            Err(e) => {
                println!("stop websocket send error {:?}", e);
            },
        }
    }

    pub fn pause(&mut self, pause: bool) {
        let result = self.send(message::pause(
            &self.guild_id.to_string(),
            pause
        ));

        match result {
            Ok(_) => {
                self.paused = pause;

                for listener in &self.listeners {
                    let handler = if pause {
                        &listener.on_player_pause
                    } else {
                        &listener.on_player_resume
                    };

                    handler(self);
                }

                println!("pause audio player: {}", pause);
            },
            Err(e) => {
                println!("pause websocket send error {:?}", e);
            },
        }
    }

    #[allow(unused)]
    pub fn seek(&mut self, position: i64) {
        unimplemented!()
    }

    pub fn volume(&mut self, volume: i32) {
        let result = self.send(message::volume(
            &self.guild_id.to_string(),
            volume
        ));

        match result {
            Ok(_) => {
                self.volume = volume;

                println!("set volume {:?}", self.volume);
            },
            Err(e) => {
                println!("play websocket send error {:?}", e);
            },
        }
    }
}

type AudioPlayerMap = HashMap<u64, Arc<Mutex<AudioPlayer>>>;

#[derive(Default)]
pub struct AudioPlayerManager {
    players: AudioPlayerMap,
}

impl AudioPlayerManager {
    // utility assosiated function for creating AudioPlayer instances wrapped in Arc & Mutex
    fn new_player(sender: NodeSender, guild_id: u64) -> Arc<Mutex<AudioPlayer>> {
        Arc::new(Mutex::new(AudioPlayer::new(sender, guild_id)))
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_player(&self, guild_id: &u64) -> bool {
        self.players.contains_key(guild_id)
    }

    pub fn get_player(&self, guild_id: &u64) -> Option<Arc<Mutex<AudioPlayer>>> {
        let player = match self.players.get(guild_id) {
            Some(player) => player,
            None => return None,
        };

        Some(Arc::clone(player)) // clone the arc
    }

    pub fn create_player(&mut self, sender: NodeSender, guild_id: u64) -> Result<Arc<Mutex<AudioPlayer>>> {
        // we dont use #has_key yet because it would get its own players clone & mutex lock
        if self.players.contains_key(&guild_id) {
            return Err(Error::PlayerAlreadyExists);
        }

        let _ = self.players.insert(guild_id, AudioPlayerManager::new_player(sender, guild_id));

        let player = &self.players[&guild_id]; // unwrap because we can assert it exists after insertion
        Ok(Arc::clone(player))
    }
}
