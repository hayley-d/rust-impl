use chrono::{DateTime, Utc};
use core::str;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fmt::Display;
use std::io::Write;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

use crate::{HttpMethod, Request};

pub struct Response {
    pub message: String,
    pub code: Code,
    pub content_type: ContentType,
    pub compression: bool,
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

impl Response {
    /// Converts the `Response` into a `Vec<u8>` suitable for sending over a TCP stream.
    pub fn to_bytes(&self) -> Vec<u8> {
        // Response line: HTTP/1.1 <status code>
        let response_line: String = format!("HTTP/1.1 {}\r\n", self.code);

        // Date Header
        let now: DateTime<Utc> = Utc::now();
        let date = now.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        let mut headers: Vec<String> = vec![
            format!("Server: Ferriscuit"),
            format!("Date: {}", date),
            format!("Cache-Control: no-cache"),
            format!("Content-Type: {}", self.content_type),
        ];

        let body: Vec<u8>;

        if !self.compression {
            body = self.message.clone().into();
        } else {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder
                .write_all(&self.message.clone().into_bytes())
                .expect("Failed to write body to gzip encoder");
            body = encoder.finish().expect("Failed to finish gzip compression");
            headers.push(format!("Content-Encoding: gzip"));
        }
        headers.push(format!("Content-Length: {}", body.len()));

        let mut response = Vec::new();
        response.extend_from_slice(response_line.as_bytes());
        response.extend_from_slice(headers.join("\r\n").as_bytes());
        response.extend_from_slice(b"\r\n\r\n");
        response.extend_from_slice(&body);

        return response;
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.code {
            Code::Ok => {
                if !self.compression {
                    write!(
                        f,
                        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                        self.code.to_code(),
                        self.code,
                        self.content_type,
                        self.message.len(),
                        self.message
                    )
                } else {
                    write!(
                        f,
                        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\n\r\n{}",
                        self.code.to_code(),
                        self.code,
                        self.content_type,
                        self.message.len(),
                        self.message
                    )
                }
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
                compression: false,
            }
        }
    };

    match request.method {
        HttpMethod::GET => handle_get(request).await,
        HttpMethod::POST => handle_post(request).await,
        _ => Response {
            message: String::from("I'm a teapot"),
            code: Code::Teapot,
            content_type: ContentType::Text,
            compression: false,
        },
    }
}

async fn handle_post(request: Request) -> Response {
    if request.uri.contains("/files/") {
    } else {
        return Response {
            message: String::from("Not found"),
            code: Code::NotFound,
            content_type: ContentType::Text,
            compression: false,
        };
    }

    let path: String = match request.get_file_path() {
        Ok(p) => p,
        Err(_) => {
            return Response {
                message: String::from("Not found"),
                code: Code::NotFound,
                content_type: ContentType::Text,
                compression: false,
            };
        }
    };

    let mut file = File::create(path).await.unwrap();

    let content =
        &request.request_headers[request.request_headers.len() - 1].trim_end_matches('\0');
    let bytes = content.as_bytes();

    match file.write_all(bytes).await {
        Ok(_) => (),
        Err(_) => {
            return Response {
                message: String::from("Internal Server Error"),
                code: Code::InternalServerError,
                content_type: ContentType::Text,
                compression: false,
            };
        }
    };

    return Response {
        message: String::from("Created file"),
        code: Code::Created,
        content_type: ContentType::Text,
        compression: false,
    };
}
async fn handle_get(request: Request) -> Response {
    if request.uri == "/" {
        return Response {
            message: String::from("OK"),
            code: Code::Ok,
            content_type: ContentType::Text,
            compression: request.is_compression_supported(),
        };
    } else if request.uri.to_lowercase().contains("echo") {
        let parts: Vec<&str> = request.uri.split("/").collect();
        let message: String = parts[parts.len() - 1].to_string();

        return Response {
            message,
            code: Code::Ok,
            content_type: ContentType::Text,
            compression: request.is_compression_supported(),
        };
    } else if request.uri.to_lowercase().contains("user-agent") {
        let user_agent: &str = request.request_headers[1]
            .split_whitespace()
            .collect::<Vec<&str>>()[1];

        return Response {
            message: user_agent.to_string(),
            code: Code::Ok,
            content_type: ContentType::Text,
            compression: request.is_compression_supported(),
        };
    } else if request.uri.to_lowercase().contains("files") {
        let path: String = match request.get_file_path() {
            Ok(p) => p,
            Err(_) => {
                return Response {
                    message: String::from("Not found"),
                    code: Code::NotFound,
                    content_type: ContentType::Text,
                    compression: false,
                };
            }
        };

        let contents: String = match fs::read_to_string(path).await {
            Ok(c) => c,
            Err(_) => {
                return Response {
                    message: String::from("Not found"),
                    code: Code::NotFound,
                    content_type: ContentType::Text,
                    compression: false,
                };
            }
        };

        return Response {
            message: contents,
            code: Code::Ok,
            content_type: ContentType::Octet,
            compression: request.is_compression_supported(),
        };
    } else {
        return Response {
            message: String::from("Not found"),
            code: Code::NotFound,
            content_type: ContentType::Text,
            compression: false,
        };
    }
}

pub enum Code {
    Ok,
    Created,
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
            Code::Created => String::from(201.to_string()),
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
            Code::Ok => write!(f, "200 OK"),
            Code::Created => write!(f, "201 Created"),
            Code::InternalServerError => write!(f, "500 Internal Server Error"),
            Code::NotFound => write!(f, "404 Not Found"),
            Code::Unauthorized => write!(f, "401 Unauthorized"),
            Code::BadRequest => write!(f, "400 Bad Request"),
            Code::Teapot => write!(f, "418 I'm a teapot"),
        }
    }
}
