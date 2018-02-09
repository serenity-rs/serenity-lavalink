use ::player::AudioPlayer;

pub trait AudioPlayerListener: Send + Sync {
    fn player_pause(&self, player: &mut AudioPlayer);
    fn player_resume(&self, player: &mut AudioPlayer);
    fn track_start(&self, player: &mut AudioPlayer, track: &str);
    fn track_end(&self, player: &mut AudioPlayer, track: &str, reason: &str);
    fn track_exception(&self, player: &mut AudioPlayer, track: &str, exception: &str);
    fn track_stuck(&self, player: &mut AudioPlayer, track: &str, threshold: i64);
}
