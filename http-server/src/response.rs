use std::env;
use std::fmt::Display;

use tokio::fs;

use crate::ServerError;

pub struct Response {
    pub message: String,
    pub code: Code,
    pub content_type: ContentType,
}

pub enum ContentType {
    Text,
    Html,
    Json,
    Octet,
}

impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentType::Text => {
                write!(f, "text/plain")
            }
            ContentType::Html => {
                write!(f, "text/html")
            }
            ContentType::Json => {
                write!(f, "application/json")
            }
            ContentType::Octet => {
                write!(f, "application/octet-stream")
            }
        }
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.code {
            Code::Ok => {
                write!(
                    f,
                    "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                    self.code.to_code(),
                    self.code,
                    self.content_type,
                    self.message.len(),
                    self.message
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

pub async fn get_response(request: String) -> Response {
    let request: Request = match Request::new(request) {
        Ok(r) => r,
        Err(_) => {
            return Response {
                message: String::from("I'm a teapot"),
                code: Code::Teapot,
                content_type: ContentType::Text,
            }
        }
    };

    if request.uri == "/" {
        return Response {
            message: String::from("OK"),
            code: Code::Ok,
            content_type: ContentType::Text,
        };
    } else if request.uri.to_lowercase().contains("echo") {
        let parts: Vec<&str> = request.uri.split("/").collect();
        let message: String = parts[parts.len() - 1].to_string();

        return Response {
            message,
            code: Code::Ok,
            content_type: ContentType::Text,
        };
    } else if request.uri.to_lowercase().contains("user-agent") {
        let user_agent: Vec<&str> = request.request[2].split_whitespace().collect();
        let user_agent: &str = user_agent[1];

        return Response {
            message: user_agent.to_string(),
            code: Code::Ok,
            content_type: ContentType::Text,
        };
    } else if request.uri.to_lowercase().contains("files") {
        let request = &request.request[0];
        let (_, uri, _) = match parse_request_line(request) {
            Ok((t, u, p)) => (t, u, p),
            Err(_) => {
                return Response {
                    message: String::from("Not found"),
                    code: Code::NotFound,
                    content_type: ContentType::Text,
                };
            }
        };
        let parts: Vec<&str> = uri.split("/").collect();

        let file_name: &str = parts[parts.len() - 1];

        let args: Vec<String> = env::args().collect();

        if args.len() < 2 {
            return Response {
                message: String::from("Not found"),
                code: Code::NotFound,
                content_type: ContentType::Text,
            };
        }

        let dir: &String = &args[2];
        let mut path: String = String::new();
        path.push_str(dir);
        path.push_str(file_name);

        let contents: String = match fs::read_to_string(path).await {
            Ok(c) => c,
            Err(_) => {
                return Response {
                    message: String::from("Not found"),
                    code: Code::NotFound,
                    content_type: ContentType::Text,
                };
            }
        };

        return Response {
            message: contents,
            code: Code::Ok,
            content_type: ContentType::Octet,
        };
    } else {
        return Response {
            message: String::from("Not found"),
            code: Code::NotFound,
            content_type: ContentType::Text,
        };
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
            Code::Ok => write!(f, "OK"),
            Code::InternalServerError => write!(f, "Internal Server Error"),
            Code::NotFound => write!(f, "Not Found"),
            Code::Unauthorized => write!(f, "Unauthorized"),
            Code::BadRequest => write!(f, "Bad Request"),
            Code::Teapot => write!(f, "I'm a teapot"),
        }
    }
}
