use std::error::Error;
use std::fmt::{self, Display};

#[derive(Debug)]
pub struct MainError {
    details: String,
}

#[derive(Debug)]
pub struct SeqError {
    details: String,
    id: String,
}

impl MainError {
    pub fn new(msg: &str) -> Self {
        MainError {
            details: String::from(msg),
        }
    }
}

impl SeqError {
    pub fn new(msg: &str, id: &[u8]) -> Self {
        let id_s = std::str::from_utf8(id).unwrap();
        SeqError {
            details: String::from(msg),
            id: String::from(id_s),
        }
    }
}

impl Display for MainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error in main thread: {}", self.details)
    }
}

impl Display for SeqError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error for sequence {}: {}", self.id, self.details)
    }
}

impl Error for MainError {}
impl Error for SeqError {}
