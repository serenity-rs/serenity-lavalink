#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;

extern crate evzht9h3nznqzwl as websocket;
extern crate hyper;
extern crate parking_lot;
extern crate percent_encoding;
extern crate serde;
extern crate serde_json;
extern crate serenity;

pub mod model;
pub mod nodes;
pub mod opcodes;
pub mod player;
pub mod rest;
pub mod stats;

mod error;
mod prelude;

pub use error::{Error, Result};
