use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    if buffer.starts_with(b"GET / HTTP/1.1") {
        format_response(
            "HTTP/1.1 200 OK",
            fs::read_to_string("html/index.html").unwrap(),
            stream,
        );
    } else {
        format_response(
            "HTTP/1.1 200 OK",
            fs::read_to_string("html/hello.html").unwrap(),
            stream,
        );
    }
}

fn format_response(status_line: &str, contents: String, mut stream: TcpStream) {
    let length: usize = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}
