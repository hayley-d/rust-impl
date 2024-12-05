use std::fmt::Display;

use crate::ServerError;

pub struct Response {
    pub message: String,
    pub code: Code,
    pub content_type: ContentType,
}

pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

pub enum ContentType {
    Text,
    Html,
    Json,
}

pub struct Request {
    pub request: Vec<String>,
    pub method: HttpMethod,
    pub uri: String,
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
        }
    }
}

impl HttpMethod {
    pub fn get_method(method: &String) -> HttpMethod {
        if method.to_uppercase().contains("GET") {
            return HttpMethod::GET;
        } else if method.to_uppercase().contains("POST") {
            return HttpMethod::POST;
        } else if method.to_uppercase().contains("PUT") {
            return HttpMethod::PUT;
        } else {
            return HttpMethod::DELETE;
        }
    }
}

impl Request {
    pub fn new(request: String) -> Result<Self, ServerError> {
        let request: Vec<String> = seporate_request(request)?;
        let (method, uri, protocol): (String, String, String) = parse_request_line(&request[0])?;
        if protocol != "HTTP/1.1" {
            return Err(ServerError {
                message: String::from("Invalid protocol"),
            });
        }
        let method: HttpMethod = HttpMethod::get_method(&method);
        return Ok(Request {
            request,
            method,
            uri,
        });
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

pub fn get_response(request: String) -> Response {
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
    } else {
        return Response {
            message: String::from("Not found"),
            code: Code::NotFound,
            content_type: ContentType::Text,
        };
    }
}

fn seporate_request(request: String) -> Result<Vec<String>, ServerError> {
    let req: Vec<String> = request.split("\r\n").map(|r| r.to_string()).collect();
    if req.len() < 3 {
        return Err(ServerError {
            message: String::from("Invalid request"),
        });
    }
    return Ok(req);
}

fn parse_request_line(request: &String) -> Result<(String, String, String), ServerError> {
    let req_line: Vec<&str> = request.split_whitespace().collect();

    if req_line.len() != 3 {
        return Err(ServerError {
            message: String::from("Invalid request line"),
        });
    }
    println!("The request method: {}", req_line[0]);
    println!("The request url: {}", req_line[1]);
    println!("The request protocol: {}", req_line[2]);

    return Ok((
        req_line[0].to_string(),
        req_line[1].to_string(),
        req_line[2].to_string(),
    ));
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
