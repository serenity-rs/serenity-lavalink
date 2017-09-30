use keys;

use lavalink::player::AudioPlayerListener;

command!(play(ctx, msg, args) {
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

    //let player_manager = data.get::<keys::LavalinkAudioPlayerManager>().expect("could not get keys::LavalinkAudioPlayerManager from Context::data");
    //let player_manager = player_manager.write().expect("could not get lock on player manager");

    let node_manager = data.get::<keys::LavalinkNodeManager>()
        .expect("could not get key::LavalinkNodeManager from Context::data")
        .read()
        .expect("could not get read lock on node_manager");

    let mut player_manager = node_manager.player_manager.write()
        .expect("could not get write lock on player manager");

    let player = if player_manager.has_player(&guild_id.0) {
        player_manager.get_player(&guild_id.0)
            .expect("audio player should be present for guild")
    } else {
        let node = match node_manager.determine_best_node() {
            Some(node) => node,
            None => {
                let _ = msg.channel_id.say("oh no! no audio processing nodes available \\:S");
                return Ok(());
            }
        };

        let player = match player_manager.create_player(node.sender.clone(), guild_id.0) {
            Ok(player) => player,
            Err(e) => {
                println!("error creating player {:?}", e);
                return Ok(());
            },
        };

        // create a new event listener for the player & register start and end listeners
        let listener = AudioPlayerListener::new()
            .with_player_pause(|player| {
                println!("paused player for guild {:?}", player.guild_id);
            })
            .with_player_resume(|player| {
                println!("resumed player for guild {:?}", player.guild_id);
            })
            .with_track_start(|player, track| {
                println!("started track {:.15}... for player of guild {:?}", track, player.guild_id);
            })
            .with_track_end(|player, track, reason| {
                println!("ended track {:.15}... for player of guild {:?}. reason: {}", track, player.guild_id, reason);
            })
            .with_track_exception(|player, track, exception| {
                println!("exception for track {:.15}... for player of guild {:?}: {}", track, player.guild_id, exception);
            })
            .with_track_stuck(|player, track, threshold_ms| {
                println!("track stuck {:.15}... for player of guild {:?}. threshold_ms: {}", track, player.guild_id, threshold_ms);
            });
        
        // register the listener
        (&player).lock().as_mut().map(|lock| lock.add_listener(listener))
            .expect("error obtaining lock on player & registering listener");

        player
    };

    player.lock().as_mut().map(|lock| lock.play(&track))
        .expect("error obtaining lock on player & calling play");
});