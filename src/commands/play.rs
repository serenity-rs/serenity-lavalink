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

    let player_exists = player_manager.has_player(&guild_id.0);

    let player = if player_exists {
        player_manager.get_player(&guild_id.0).expect("audio player should be present for guild")
    } else {
        let mut player_manager = player_manager;
        let ws_tx = data.get::<keys::LavalinkSocketSender>().unwrap().clone();

        let player = match player_manager.create_player(ws_tx, guild_id.0) {
            Ok(player) => player,
            Err(e) => {
                println!("error creating player {:?}", e);
                return Ok(());
            }
        };

        // create a new event listener for the player
        let mut listener = AudioPlayerListener::new();

        // listen for the track start event
        listener.on_track_start = |player, track| {
            println!("started track {} for player of guild {:?}", track, player.guild_id);
        };

        // listen for the track end event
        listener.on_track_end = |player, track, reason| {
            println!("ended track {} for player of guild {:?}. reason: {}", track, player.guild_id, reason);
        };
        
        {
            let player = player.clone(); // clone the arc (again)
            let mut player = player.lock().unwrap();

            // register the listener
            player.add_listener(listener);
        }

        player
    };

    let mut player = player.lock().unwrap();

    // play the track :)
    player.play(track);

    Ok(())
}