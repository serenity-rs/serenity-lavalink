use hyper::Error as HyperError;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::result::Result as StdResult;
use std::sync::mpsc::SendError;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    Hyper(HyperError),
    PlayerAlreadyExists,
    Send(String),
    StatsNotPresent,
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
            Error::PlayerAlreadyExists => "Player already exists for the guild",
            Error::Send(ref inner) => inner,
            Error::StatsNotPresent => "No stats are present",
        }
    }
}

impl From<HyperError> for Error {
    fn from(err: HyperError) -> Self {
        Error::Hyper(err)
    }
}

impl<T> From<SendError<T>> for Error {
    fn from(err: SendError<T>) -> Self {
        Error::Send(format!("{}", err))
    }
}
