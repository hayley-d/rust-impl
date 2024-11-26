use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::{
    fs,
    io::{prelude::*, BufReader, Error},
    net::{Ipv6Addr, SocketAddr, SocketAddrV6, TcpListener, TcpStream},
};
use synchronous_server::my_errors::{self, SocketError};

fn main() -> Result<(), SocketError> {
    //let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
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

fn create_socket() -> Result<Socket, SocketError> {
    let socket = match Socket::new(Domain::IPV6, Type::STREAM, Some(Protocol::TCP)) {
        Ok(s) => s,
        Err(_) => {
            let error = SocketError {
                msg: String::from("Error creating socket"),
            };
            eprintln!("{}", error);
            return Err(error);
        }
    };

    match socket.set_reuse_address(true) {
        Ok(_) => (),
        Err(_) => {
            let error = SocketError {
                msg: String::from("Error setting resuse address"),
            };
            eprintln!("{}", error);
            return Err(error);
        }
    };

    let socket_address = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 7878, 0, 0);

    assert_eq!("[::1]:7878".parse(), Ok(socket_address));
    assert_eq!(socket_address.ip(), &Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
    assert_eq!(socket_address.port(), 7878);

    let socket_address = SockAddr::from(socket_address);
    match socket.bind(&socket_address) {
        Ok(_) => (),
        Err(_) => {
            let error = SocketError {
                msg: String::from("Error binding address to socket"),
            };
            eprintln!("{}", error);
            return Err(error);
        }
    };

    match socket.listen(128) {
        Ok(_) => (),
        Err(_) => {
            let error = SocketError {
                msg: String::from("Error binding address to socket"),
            };
            eprintln!("{}", error);
            return Err(error);
        }
    };

    println!("Listening on [::1]:7878...");

    return Ok(socket);
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
