use std::env;

use crate::ServerError;

pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

pub struct Request {
    pub request_headers: Vec<String>,
    pub method: HttpMethod,
    pub uri: String,
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

impl PartialEq for HttpMethod {
    fn eq(&self, other: &Self) -> bool {
        match self {
            HttpMethod::GET => match other {
                HttpMethod::GET => true,
                _ => false,
            },
            HttpMethod::POST => match other {
                HttpMethod::POST => true,
                _ => false,
            },
            HttpMethod::PUT => match other {
                HttpMethod::PUT => true,
                _ => false,
            },
            HttpMethod::DELETE => match other {
                HttpMethod::DELETE => true,
                _ => false,
            },
        }
    }
}

impl Request {
    pub fn new(request: String) -> Result<Self, ServerError> {
        let request: Vec<String> = Self::seporate_request(request)?;
        let (method, uri, protocol): (String, String, String) =
            Self::parse_request_line(&request[0])?;

        if protocol != "HTTP/1.1" {
            return Err(ServerError {
                message: String::from("Invalid protocol"),
            });
        }

        let method: HttpMethod = HttpMethod::get_method(&method);

        return Ok(Request {
            request_headers: request[1..].to_vec(),
            method,
            uri,
        });
    }

    // Takes in a full request and seporates it into parts
    fn seporate_request(request: String) -> Result<Vec<String>, ServerError> {
        let request_length: usize = request.split("\r\n").count();

        // if the length is less then it is invalid
        if request_length < 3 {
            return Err(ServerError {
                message: String::from("Invalid request"),
            });
        }

        let mut req: Vec<String> = Vec::with_capacity(request_length);

        for r in request.split("\r\n") {
            let temp: String = r.to_string().trim().to_string();
            println!("{}", temp);
            req.push(temp);
        }

        return Ok(req);
    }

    fn parse_request_line(request: &String) -> Result<(String, String, String), ServerError> {
        let req_line: [&str; 3] = match request.split_whitespace().collect::<Vec<&str>>().try_into()
        {
            Ok(s) => s,
            Err(_) => {
                return Err(ServerError {
                    message: String::from("Invalid request line"),
                });
            }
        };

        return Ok((
            req_line[0].to_string(),
            req_line[1].to_string(),
            req_line[2].to_string(),
        ));
    }

    pub fn get_file_path(&self) -> Result<String, ServerError> {
        let parts: Vec<&str> = self.uri.split("/").collect();

        let file_name: &str = parts[parts.len() - 1];

        let args: Vec<String> = env::args().collect();

        if args.len() < 2 {
            return Err(ServerError {
                message: String::from("Invalid command line arguments"),
            });
        }

        let dir: &String = &args[2];
        let mut path: String = String::new();
        path.push_str(dir);
        path.push_str(file_name);

        return Ok(path);
    }

    pub fn is_compression_supported(&self) -> bool {
        for header in &self.request_headers {
            if header.to_lowercase().contains("accept-encoding") {
                if header
                    .to_lowercase()
                    .split_whitespace()
                    .collect::<Vec<&str>>()[1]
                    == "gzip"
                {
                    return true;
                }
            }
        }
        return false;
    }
}
