use std::fmt::Display;

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl Clone for Error {
    fn clone(&self) -> Self {
        return Error {
            message: self.message.clone(),
        };
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}
