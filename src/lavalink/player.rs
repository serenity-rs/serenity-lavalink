use serenity::model::GuildId;

use std::collections::HashMap;
use std::marker::Send;
use std::sync::{Arc, Mutex};

type PlayerPauseHandler = Fn(&AudioPlayer) + Send + Sync;
type PlayerResumeHandler = Fn(&AudioPlayer) + Send + Sync;
type TrackStartHandler = Fn(&AudioPlayer, String) + Send + Sync;
type TrackEndHandler = Fn(&AudioPlayer, String, String) + Send + Sync;
type TrackExceptionHandler = Fn(&AudioPlayer, String, String) + Send + Sync;
type TrackStuckHandler = Fn(&AudioPlayer, String, i64) + Send + Sync;

pub struct AudioPlayerListener {
    on_player_pause: Arc<Option<Box<PlayerPauseHandler>>>,
    on_player_resume: Arc<Option<Box<PlayerResumeHandler>>>,
    on_track_start: Arc<Option<Box<TrackStartHandler>>>,
    on_track_end: Arc<Option<Box<TrackEndHandler>>>,
    on_track_exception: Arc<Option<Box<TrackExceptionHandler>>>,
    on_track_stuck: Arc<Option<Box<TrackStuckHandler>>>,
}

impl AudioPlayerListener {
    pub fn new() -> Self {
        Self {
            // disgusting..
            on_player_pause: Arc::new(None),
            on_player_resume: Arc::new(None),
            on_track_start: Arc::new(None),
            on_track_end: Arc::new(None),
            on_track_exception: Arc::new(None),
            on_track_stuck: Arc::new(None),
        }
    }

    pub fn on_player_pause(&mut self, callback: Box<PlayerPauseHandler>) -> &mut Self {
        self.on_player_pause = Arc::new(Some(callback));
        self
    }
}

pub struct AudioPlayer {
    pub track: Option<String>,
    pub time: i64,
    pub position: i64,
    pub paused: bool,
    listeners: Vec<AudioPlayerListener>,
}

impl AudioPlayer {
    fn new() -> Self {
        Self {
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
}

type AudioPlayerMap = HashMap<GuildId, Arc<Mutex<AudioPlayer>>>;

pub struct AudioPlayerManager {
    players: AudioPlayerMap,
}

impl AudioPlayerManager {
    // utility assosiated function for creating AudioPlayer instances wrapped in Arc & Mutex
    fn new_player() -> Arc<Mutex<AudioPlayer>> {
        Arc::new(Mutex::new(AudioPlayer::new()))
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

    pub fn create_player(&mut self, guild_id: GuildId) -> Result<Arc<Mutex<AudioPlayer>>, String> {
        // we dont use #has_key yet because it would get its own players clone & mutex lock
        if self.players.contains_key(&guild_id) {
            return Err(format!("player already exists under the guild id {}", &guild_id));
        }

        let _ = self.players.insert(guild_id.clone(), AudioPlayerManager::new_player());
        
        let player = self.players.get(&guild_id).unwrap(); // unwrap because we can assert it exists after insertion
        Ok(player.clone())
    }
}