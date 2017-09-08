use keys::LavalinkConfig;
use ::lavalink::rest;

use serenity::model::*;
use serenity::client::Context;
use serenity::framework::standard::Args;

pub fn search(ctx: &mut Context, msg: &Message, args: Args) -> Result<(), String> {
    let identifier = match args.list::<String>() {
        Ok(list) => list.join(" "),
        Err(_) => {
            let _ = msg.channel_id.say("usage: !search <identifier>");
            return Ok(());
        }
    };

    let data = ctx.data.lock();
    let config = data.get::<LavalinkConfig>().unwrap();

    // haha yes this is incredibly inefficient sorry tokio :'(
    let mut http_client = rest::HttpClient::new(&config);
    let tracks = http_client.load_tracks(&identifier);

    let response = tracks.into_iter()
        .enumerate()
        .take_while(|e| e.0 < 5)
        .map(|e| e.1.info.title)
        .collect::<Vec<String>>()
        .join("\n");

    let _ = msg.channel_id.say(response);

    Ok(())
}