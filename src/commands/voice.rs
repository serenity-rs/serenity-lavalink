use keys;

use lavalink::message;

command!(join(ctx, msg) {
    let guild = match msg.guild() {
        Some(guild) => guild,
        None => {
            println!("None of them guilds :((");
            return Ok(());
        },
    };
    
    let guild = guild.read().unwrap();
    let guild_id = guild.id.0;
    let user_id = msg.author.id;
    
    let voice_state = match guild.voice_states.get(&user_id) {
        Some(voice_state) => voice_state,
        None => {
            let _ = msg.channel_id.say("You must be in a voice channel....");
            return Ok(());
        },
    };

    let vs_channel_id = &voice_state.channel_id.unwrap().0;
    let _ = msg.channel_id.say(format!("Joining voice channel <#{}> ({})", vs_channel_id, vs_channel_id));
    
    let data = ctx.data.lock();
    //let ws_tx = data.get::<keys::LavalinkSocketSender>().unwrap().clone();

    //let _ = ws_tx.lock().unwrap().send(message::connect(&guild.id.0.to_string(), &voice_state.channel_id.unwrap().to_string()));

    let node_manager = data.get::<keys::LavalinkNodeManager>()
        .expect("could not get key::LavalinkNodeManager from Context::data")
        .read()
        .expect("could not get read lock on node_manager");

    let player_manager = node_manager.player_manager.read()
            .expect("could not get write lock on player manager");

    let player = match player_manager.get_player(&guild_id) {
        Some(player) => player,
        None => {
            panic!("got voice server update for guild {} without player", &guild_id);
            return Ok(());
        }
    };

    let reader = player.lock().expect("could not get read lock on player");
    let sender = reader.sender.lock().expect("could not get lock on node sender");
    let json_data = message::connect(&guild.id.0.to_string(), &voice_state.channel_id.unwrap().to_string());

    let _ = sender.send(json_data)
        .map_err(|e| panic!("error sending json data: {:?}", e));
});

command!(leave(ctx, msg) {
    let guild_id = match msg.guild_id() {
        Some(guild_id) => guild_id,
        None => {
            println!("oh no! no guild id??");
            return Ok(());
        },
    };

    let guild_id_u64 = guild_id.0;
    let guild_id_str = guild_id.to_string();

    let _ = msg.channel_id.say("leaving channel");
    
    let data = ctx.data.lock();
    //let ws_tx = data.get::<keys::LavalinkSocketSender>().unwrap().clone();

    //let _ = ws_tx.lock().unwrap().send(message::disconnect(&guild_id));

    let node_manager = data.get::<keys::LavalinkNodeManager>()
        .expect("could not get key::LavalinkNodeManager from Context::data")
        .read()
        .expect("could not get read lock on node_manager");

    let player_manager = node_manager.player_manager.read()
            .expect("could not get write lock on player manager");

    let player = match player_manager.get_player(&guild_id_u64) {
        Some(player) => player,
        None => {
            panic!("got voice server update for guild {} without player", &guild_id);
            return Ok(());
        }
    };

    let reader = player.lock().expect("could not get read lock on player");
    let sender = reader.sender.lock().expect("could not get lock on node sender");
    let json_data = message::disconnect(&guild_id_str);

    let _ = sender.send(json_data)
        .map_err(|e| panic!("error sending json data: {:?}", e));
});