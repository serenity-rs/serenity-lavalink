use serenity::voice;
use serenity::model::*;
use serenity::prelude::*;

pub struct Handler;

impl EventHandler for Handler {
    fn on_ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    fn on_message(&self, _: Context, message: Message) {
        println!("got msg: {}", message.content);
    }
}