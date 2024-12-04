use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub struct ServerError {
    pub message: String,
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl Error for ServerError {}
