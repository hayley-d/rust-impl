use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::{
    fs,
    io::{prelude::*, BufReader, Error},
    net::{Ipv6Addr, SocketAddr, SocketAddrV6, TcpListener, TcpStream},
};
use synchronous_server::{my_errors::SocketError, my_socket::create_socket};

fn main() -> Result<(), SocketError> {
    let socket = create_socket()?;
    let listener: TcpListener = socket.into();

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(_) => {
                let error = SocketError {
                    msg: String::from("Error creating socket"),
                };
                eprintln!("{}", error);
                return Err(error);
            }
        };

        handle_connection(stream);
    }

    Ok(())
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
