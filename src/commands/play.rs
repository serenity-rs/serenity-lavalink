use keys;

use lavalink::player::AudioPlayerListener;
use serenity::client::Context;
use serenity::framework::standard::Args;
use serenity::model::*;

pub fn play(ctx: &mut Context, msg: &Message, args: Args) -> Result<(), String> {
    let track = match args.clone().single::<String>() {
        Ok(track) => track,
        Err(_) => {
            let _ = msg.channel_id.say("usage: !play <encoded track>");
            return Ok(());
        },
    };

    let guild_id = match msg.guild_id() {
        Some(guild_id) => guild_id,
        None => {
            println!("oh no! no guild id??");
            return Ok(());
        },
    };
    
    let data = ctx.data.lock();

    let player_manager = data.get::<keys::LavalinkAudioPlayerManager>().expect("could not get key::LavalinkAudioPlayerManager from Context::data");
    let player_manager = player_manager.write().expect("could not get lock on player manager");

    let player = if player_manager.has_player(&guild_id.0) {
        player_manager.get_player(&guild_id.0)
            .expect("audio player should be present for guild")
    } else {
        let mut player_manager = player_manager;

        let ws_tx = data.get::<keys::LavalinkSocketSender>()
            .expect("could not get key::LavalinkSocketSender from Context::data");

        let player = match player_manager.create_player(ws_tx.clone(), guild_id.0) {
            Ok(player) => player,
            Err(e) => {
                println!("error creating player {:?}", e);
                return Ok(());
            },
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
        
        // register the listener
        (&player).lock().as_mut().map(|lock| lock.add_listener(listener))
            .expect("error obtaining lock on player & registering listener");

        player
    };

    player.lock().as_mut().map(|lock| lock.play(&track))
        .expect("error obtaining lock on player & calling play");

    Ok(())
}