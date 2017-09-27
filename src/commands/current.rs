use keys;

command!(current(ctx, msg) {
    let guild_id = match msg.guild_id() {
        Some(guild_id) => guild_id,
        None => {
            println!("oh no! no guild id??");
            return Ok(());
        },
    };

    let data = ctx.data.lock();

    let player_manager = data.get::<keys::LavalinkAudioPlayerManager>()
        .expect("keys::LavalinkAudioPlayerManager is not present in Context::data").clone();

    let player_manager = player_manager.read().unwrap();

    if !player_manager.has_player(&guild_id.0) {
        let _ = msg.channel_id.say("this channel does not have an audio player");
        return Ok(());
    }

    let player = player_manager.get_player(&guild_id.0)
        .expect("audio player should be present for guild");
        
    let player = player.lock()
        .expect("could not access mutex for player");

    let _ = msg.channel_id.say(&format!(
        "track: {:?}\nposition/time: {}/{}\npaused: {}\nvolume: {}", 
        &player.track, player.position, player.time, player.paused, player.volume
    ));
});