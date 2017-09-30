use keys;

command!(stop(ctx, msg) {
    let guild_id = match msg.guild_id() {
        Some(guild_id) => guild_id.0,
        None => {
            println!("oh no! no guild id??");
            return Ok(());
        },
    };

    {
        let data = ctx.data.lock();

        let node_manager = data.get::<keys::LavalinkNodeManager>()
            .expect("could not get key::LavalinkNodeManager from Context::data")
            .read()
            .expect("could not get read lock on node_manager");

        let player_manager = node_manager.player_manager.read()
            .expect("could not get write lock on player manager");

        let player = match player_manager.get_player(&guild_id) {
            Some(player) => player,
            None => {
                let _ = msg.channel_id.say("this guild does not have an audio player");
                return Ok(());
            }
        };

        player.lock().as_mut().map(|lock| lock.stop())
            .expect("error obtaining lock on player and stopping");
    }

    let _ = msg.channel_id.say("stopped playing!");
});