use super::message;
use super::socket::SocketSender;

use serenity::model::GuildId;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::SendError;

use websocket::OwnedMessage;

type PlayerPauseHandler = fn(&AudioPlayer);
type PlayerResumeHandler = fn(&AudioPlayer);
type TrackStartHandler = fn(&AudioPlayer, String);
type TrackEndHandler = fn(&AudioPlayer, String, String);
type TrackExceptionHandler = fn(&AudioPlayer, String, String);
type TrackStuckHandler = fn(&AudioPlayer, String, i64);

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
        Self {
            // disgusting..
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
    pub sender: SocketSender,
    pub guild_id: GuildId,
    pub track: Option<String>,
    pub time: i64,
    pub position: i64,
    pub paused: bool,
    pub volume: i32,
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
            volume: 100,
            listeners: Vec::new(),
        }
    }

    pub fn add_listener(&mut self, listener: AudioPlayerListener) {
        self.listeners.push(listener);
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

    pub fn stop(&mut self) {
        let result = self.send(message::stop(
            &self.guild_id.0.to_string()
        ));
        
        match result {
            Ok(_) => {
                let track = self.track.clone().unwrap_or("no track in state".to_string());
                self.track = None;

                for listener in &self.listeners {
                    let on_track_end = &listener.on_track_end;
                    on_track_end(self, track.to_string(), "no reason :) :dabs:".to_string());
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
            &self.guild_id.0.to_string(), 
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
            &self.guild_id.0.to_string(), 
            volume
        ));
        
        match result {
            Ok(_) => {
                self.volume = volume;

                println!("started playing track {:?}", self.track);
            },
            Err(e) => {
                println!("play websocket send error {:?}", e);
            },
        }
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