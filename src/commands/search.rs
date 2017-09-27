use keys::LavalinkConfig;

use lavalink::rest;
use serenity::client::Context;
use serenity::framework::standard::Args;
use serenity::model::*;

pub fn search(ctx: &mut Context, msg: &Message, args: Args) -> Result<(), String> {
    let identifier = match args.list::<String>() {
        Ok(list) => list.join(" "),
        Err(_) => {
            let _ = msg.channel_id.say("usage: !search <identifier>");
            return Ok(());
        },
    };

    let data = ctx.data.lock();
    let config = data.get::<LavalinkConfig>().unwrap();

    let http_client = rest::HttpClient::new(&config);
    let tracks = http_client.load_tracks(&identifier).expect("could not load tracks");

    if tracks.len() == 0 {
        let _ = msg.channel_id.say("No results found!");
        return Ok(());
    }

    let response = tracks.into_iter()
        .enumerate()
        .take_while(|e| e.0 < 3)
        .map(|e| format!("{}:\n`{}`", e.1.info.title, e.1.track))
        .collect::<Vec<String>>()
        .join("\n\n");

    let _ = msg.channel_id.say(response);

    Ok(())
}