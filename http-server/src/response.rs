use std::error::Error;
use std::fmt::Display;

pub struct Response {
    pub message: String,
    pub code: Code,
}

impl Response {}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.code {
            Code::Ok => {
                write!(
                    f,
                    "HTTP/1.1 {} {}\r\n\r\n",
                    self.code.to_code(),
                    self.code.to_string()
                )
            }
            _ => {
                write!(
                    f,
                    "HTTP/1.1 {} {}\r\n\r\n{}\r\n\r\n",
                    self.code.to_code(),
                    self.code.to_string(),
                    self.message,
                )
            }
        }
    }
}

pub enum Code {
    Ok,
    InternalServerError,
    Unauthorized,
    NotFound,
    BadRequest,
    Teapot,
}

impl Code {
    pub fn to_code(&self) -> String {
        match self {
            Code::Ok => String::from(200.to_string()),
            Code::InternalServerError => String::from(500.to_string()),
            Code::Unauthorized => String::from(401.to_string()),
            Code::NotFound => String::from(404.to_string()),
            Code::BadRequest => String::from(400.to_string()),
            Code::Teapot => String::from(418.to_string()),
        }
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Code::Ok => write!(f, "Ok"),
            Code::InternalServerError => write!(f, "Internal Server Error"),
            Code::NotFound => write!(f, "Not Found"),
            Code::Unauthorized => write!(f, "Unauthorized"),
            Code::BadRequest => write!(f, "Bad Request"),
            Code::Teapot => write!(f, "I'm a teapot"),
        }
    }
}
