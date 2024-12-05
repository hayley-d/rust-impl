pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

pub struct Request {
    pub request: Vec<String>,
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

fn seporate_request(request: String) -> Result<Vec<String>, ServerError> {
    let req: Vec<String> = request.split("\r\n").map(|r| r.to_string()).collect();
    let length: usize = req.len();

    if length < 3 {
        return Err(ServerError {
            message: String::from("Invalid request"),
        });
    }

    let mut request: Vec<String> = Vec::with_capacity(length);

    for r in req {
        request.push(r);
    }

    return Ok(request);
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
