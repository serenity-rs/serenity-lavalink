use keys;

use lavalink::player::AudioPlayerListener;

use serenity::model::*;
use serenity::client::Context;
use serenity::framework::standard::Args;

pub fn play(ctx: &mut Context, msg: &Message, args: Args) -> Result<(), String> {
    let args = match args.list::<String>() {
        Ok(args) => args,
        Err(_) => {
            let _ = msg.channel_id.say("usage: !play <encoded track>");
            return Ok(());
        },
    };
    let track = args.get(0).unwrap();

    let guild_id = match msg.guild_id() {
        Some(guild_id) => guild_id,
        None => {
            println!("oh no! no guild id??");
            return Ok(());
        },
    };
    
    let data = ctx.data.lock();

    let player_manager = data.get::<keys::LavalinkAudioPlayerManager>().unwrap().clone();
    let player_manager = player_manager.lock().unwrap();

    let player_exists = player_manager.has_player(&guild_id);

    let player = if player_exists {
        player_manager.get_player(&guild_id).expect("audio player should be present for guild")
    } else {
        let mut player_manager = player_manager;
        let ws_tx = data.get::<keys::LavalinkSocketSender>().unwrap().clone();

        match player_manager.create_player(ws_tx, guild_id) {
            Ok(player) => player,
            Err(e) => {
                println!("error creating player {:?}", e);
                return Ok(());
            }
        }
    };
    let mut player = player.lock().unwrap();

    player.play(track);

    Ok(())
}