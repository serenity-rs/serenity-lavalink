use super::message;
use super::socket::SocketSender;

use serenity::model::GuildId;

use std::collections::HashMap;
use std::marker::Send;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, SendError};

use websocket::OwnedMessage;

type PlayerPauseHandler = Fn(&AudioPlayer) + Send + Sync;
type PlayerResumeHandler = Fn(&AudioPlayer) + Send + Sync;
type TrackStartHandler = Fn(&AudioPlayer, String) + Send + Sync;
type TrackEndHandler = Fn(&AudioPlayer, String, String) + Send + Sync;
type TrackExceptionHandler = Fn(&AudioPlayer, String, String) + Send + Sync;
type TrackStuckHandler = Fn(&AudioPlayer, String, i64) + Send + Sync;

pub struct AudioPlayerListener {
    pub on_player_pause: Box<PlayerPauseHandler>,
    pub on_player_resume: Box<PlayerResumeHandler>,
    pub on_track_start: Box<TrackStartHandler>,
    pub on_track_end: Box<TrackEndHandler>,
    pub on_track_exception: Box<TrackExceptionHandler>,
    pub on_track_stuck: Box<TrackStuckHandler>,
}

impl AudioPlayerListener {
    pub fn new() -> Self {
        Self {
            // disgusting..
            on_player_pause: Box::new(|_| {}),
            on_player_resume: Box::new(|_| {}),
            on_track_start: Box::new(|_, _| {}),
            on_track_end: Box::new(|_, _, _| {}),
            on_track_exception: Box::new(|_, _, _| {}),
            on_track_stuck: Box::new(|_, _, _| {}),
        }
    }
}

// todo potentially split state into child struct to avoid mutable reference of AudioPlayer
// where mutablity should not be nessesary for non state fields
pub struct AudioPlayer {
    pub sender: SocketSender,
    pub guild_id: GuildId,
    pub track: Option<String>,
    pub time: i64,
    pub position: i64,
    pub paused: bool,
    listeners: Vec<AudioPlayerListener>,
}

impl AudioPlayer {
    fn new(sender: SocketSender, guild_id: GuildId) -> Self {
        Self {
            sender,
            guild_id,
            track: None,
            time: 0,
            position: 0,
            paused: false,
            listeners: Vec::new(),
        }
    }

    pub fn add_listener(&mut self, listener: AudioPlayerListener) {
        self.listeners.push(AudioPlayerListener::new());
    }

    fn send(&self, message: OwnedMessage) -> Result<(), SendError<OwnedMessage>> {
        self.sender.lock().unwrap().send(message)
    }

    pub fn play(&mut self, track: &str) {
        let result = self.send(message::play(
            &self.guild_id.0.to_string(), 
            track
        ));
        
        match result {
            Ok(_) => {
                self.track = Some(track.to_string());
                
                for listener in &self.listeners {
                    let on_track_start = &listener.on_track_start;
                    on_track_start(self, track.to_string());
                }

                println!("started playing track {:?}", self.track);
            },
            Err(e) => {
                println!("play websocket send error {:?}", e);
            },
        }
    }

    pub fn stop(&self) {
        unimplemented!()
    }

    pub fn pause(&self, pause: bool) {
        unimplemented!()
    }

    pub fn seek(&self, position: i64) {
        unimplemented!()
    }

    pub fn volume(&self, volume: i32) {
        unimplemented!()
    }
}

type AudioPlayerMap = HashMap<GuildId, Arc<Mutex<AudioPlayer>>>;

pub struct AudioPlayerManager {
    players: AudioPlayerMap,
}

impl AudioPlayerManager {
    // utility assosiated function for creating AudioPlayer instances wrapped in Arc & Mutex
    fn new_player(sender: SocketSender, guild_id: GuildId) -> Arc<Mutex<AudioPlayer>> {
        Arc::new(Mutex::new(AudioPlayer::new(sender, guild_id)))
    }
    
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }

    pub fn has_player(&self, guild_id: &GuildId) -> bool {
        self.players.contains_key(&guild_id)
    }

    pub fn get_player(&self, guild_id: &GuildId) -> Option<Arc<Mutex<AudioPlayer>>> {
        let player = match self.players.get(guild_id) {
            Some(player) => player,
            None => return None,
        };

        Some(player.clone()) // clone the arc
    }

    pub fn create_player(&mut self, sender: SocketSender, guild_id: GuildId) -> Result<Arc<Mutex<AudioPlayer>>, String> {
        // we dont use #has_key yet because it would get its own players clone & mutex lock
        if self.players.contains_key(&guild_id) {
            return Err(format!("player already exists under the guild id {}", &guild_id));
        }

        let _ = self.players.insert(guild_id.clone(), AudioPlayerManager::new_player(sender, guild_id.clone()));
        
        let player = self.players.get(&guild_id).unwrap(); // unwrap because we can assert it exists after insertion
        Ok(player.clone())
    }
}