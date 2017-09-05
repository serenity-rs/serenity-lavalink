pub mod admin;
pub mod meta;
pub mod search;
pub mod voice;

use lavalink::config::Config;

use typemap::Key;

pub struct LavalinkConfigKey;

impl Key for LavalinkConfigKey {
    type Value = Config;
}