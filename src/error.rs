use hyper::Error as HyperError;
use serde_json::Error as JsonError;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Error as IoError;
use std::result::Result as StdResult;
use std::sync::mpsc::SendError;
use websocket::client::ParseError;
use websocket::WebSocketError;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    Hyper(HyperError),
    Io(IoError),
    Json(JsonError),
    PlayerAlreadyExists,
    Send(String),
    StatsNotPresent,
    UriParse(ParseError),
    WebSocket(WebSocketError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Hyper(ref inner) => inner.description(),
            Error::Io(ref inner) => inner.description(),
            Error::Json(ref inner) => inner.description(),
            Error::PlayerAlreadyExists => "Player already exists for the guild",
            Error::Send(ref inner) => inner,
            Error::StatsNotPresent => "No stats are present",
            Error::UriParse(ref inner) => inner.description(),
            Error::WebSocket(ref inner) => inner.description(),
        }
    }
}

impl From<HyperError> for Error {
    fn from(err: HyperError) -> Self {
        Error::Hyper(err)
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Error::Io(err)
    }
}

impl From<JsonError> for Error {
    fn from(err: JsonError) -> Self {
        Error::Json(err)
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Error::UriParse(err)
    }
}

impl<T> From<SendError<T>> for Error {
    fn from(err: SendError<T>) -> Self {
        Error::Send(format!("{}", err))
    }
}

impl From<WebSocketError> for Error {
    fn from(err: WebSocketError) -> Self {
        Error::WebSocket(err)
    }
}
