#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;

extern crate evzht9h3nznqzwl as websocket;
extern crate hyper;
extern crate parking_lot;
extern crate percent_encoding;
extern crate serde;
extern crate serenity;

pub mod message;
pub mod node;
pub mod opcodes;
pub mod player;
pub mod rest;
pub mod stats;

mod error;
mod prelude;

pub use error::{Error, Result};
