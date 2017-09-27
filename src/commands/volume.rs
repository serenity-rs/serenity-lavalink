use keys;

const INVALID_ARGUMENTS_MESSAGE: &'static str = "usage: `!volume <1 to 150>` (default 100)";

command!(volume(ctx, msg, args) {
    let volume = match args.clone().single::<u8>() {
        Ok(volume) => volume,
        Err(_) => {
            let _ = msg.channel_id.say(INVALID_ARGUMENTS_MESSAGE);
            return Ok(());
        },
    };

    if volume < 1 || volume > 150 {
        let _ = msg.channel_id.say(INVALID_ARGUMENTS_MESSAGE);
        return Ok(());
    }

    let guild_id = match msg.guild_id() {
        Some(guild_id) => guild_id.0,
        None => {
            println!("oh no! no guild id??");
            return Ok(());
        },
    };

    {
        let data = ctx.data.lock();

        let player_manager = data.get::<keys::LavalinkAudioPlayerManager>()
            .expect("keys::LavalinkAudioPlayerManager not present in Context::data");

        let player_manager = player_manager.read()
            .expect("could not obtain lock on player manager");

        let player = match player_manager.get_player(&guild_id) {
            Some(player) => player,
            None => {
                let _ = msg.channel_id.say("this guild does not have an audio player");
                return Ok(());
            },
        };

        player.lock().as_mut().map(|lock| lock.volume(volume as i32))
            .expect("error obtaining lock on player and changing volume");
    }

    let _ = msg.channel_id.say(&format!("changed volume to {}/150", volume));
});