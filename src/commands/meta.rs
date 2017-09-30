use keys;

use lavalink::stats::RemoteStats;
use serenity::client::Context;

command!(ping(ctx, msg) {
    let _ = if let Some(latency) = ctx.shard.lock().latency() {
        msg.channel_id.say(
            format!("Pong! Shard gateway heartbeat latency: {}.{}s",
                latency.as_secs(), latency.subsec_nanos()))
    } else {
        msg.channel_id.say("Pong! No latency recorded!")
    };
});

fn get_socket_stats(ctx: &mut Context) -> Result<Vec<RemoteStats>, &'static str> {
    // note when more stats functions are added this should be passed between them
    // instead of obtaining a data lock for each function
    let data = ctx.data.lock();

    let node_manager = data.get::<keys::LavalinkNodeManager>()
        .expect("could not get key::LavalinkNodeManager from Context::data")
        .read()
        .expect("could not get read lock on node_manager");

    let stats_vec = Vec::new();

    for node in node_manager.nodes.read().expect("could not get read lock on nodes").iter() {
        let socket_state = node.state.read().expect("could not get read lock on socket state");
        
        match socket_state.stats.clone() {
            Some(stats) => stats_vec.push(stats),
            None => {
                return Err("no socket stats are available yet");
            },
        }
    }

    Ok(stats_vec)
}

command!(stats(ctx, msg) {
    let socket_stats = get_socket_stats(ctx); // as well as looking cleaner this should reduce the scope of the locks

    let _ = msg.channel_id.say(
        &format!("lavalink node:```\n{:?}\n```", socket_stats)
    );
});