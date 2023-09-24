use std::{fmt, error::Error};


#[derive(Debug)]
pub enum MinerError {
    // Errors from external libraries...
    Io(std::io::Error),
    Reqwest(reqwest::Error),
    Regex(regex::Error),
    Serde(serde_json::Error),
    // Errors raised by us...
    Message(String)
}

pub fn message(msg :&str) -> MinerError {
    MinerError::Message(msg.to_string())
}

impl Error for MinerError {
    fn description(&self) -> &str {
        match *self {
            MinerError::Io(ref err) => err.description(),
            MinerError::Regex(ref err) => err.description(),
            MinerError::Reqwest(ref err) => err.description(),
            MinerError::Serde(ref err) => err.description(),
            MinerError::Message(ref s) => &s,
        }
    }
}

impl fmt::Display for MinerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MinerError::Io(ref err) => err.fmt(f),
            MinerError::Reqwest(ref err) => err.fmt(f),
            MinerError::Regex(ref err) => err.fmt(f),
            MinerError::Serde(ref err) => err.fmt(f),
            MinerError::Message(ref err) => err.fmt(f),
        }
    }
}

impl From<std::io::Error> for MinerError {
    fn from(err: std::io::Error) -> MinerError {
        MinerError::Io(err)
    }
}

impl From<reqwest::Error> for MinerError {
    fn from(err: reqwest::Error) -> MinerError {
        MinerError::Reqwest(err)
    }
}

impl From<regex::Error> for MinerError {
    fn from(err: regex::Error) -> MinerError {
        MinerError::Regex(err)
    }
}

impl From<serde_json::Error> for MinerError {
    fn from(err: serde_json::Error) -> MinerError {
        MinerError::Serde(err)
    }
}

impl From<String> for MinerError {
    fn from(msg: String) -> MinerError {
        MinerError::Message(msg)
    }
}

pub type Result<T> = std::result::Result<T, MinerError>;