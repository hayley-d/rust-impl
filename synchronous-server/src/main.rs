use std::{
    fs,
    io::prelude::*,
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};
use synchronous_server::{
    my_errors::SocketError, my_socket::create_socket, my_threadpool::ThreadPool,
};

fn main() -> Result<(), SocketError> {
    let socket = create_socket()?;
    let listener: TcpListener = socket.into();
    let thread_pool: ThreadPool = ThreadPool::new(5)?;

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

        thread_pool.execute(|| {
            handle_connection(stream);
        });
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let home_route: &[u8] = b"GET / HTTP/1.1";
    let hayley_route: &[u8] = b"GET /hayley HTTP/1.1";

    if buffer.starts_with(home_route) {
        format_response(
            "HTTP/1.1 200 OK",
            fs::read_to_string("html/index.html").unwrap(),
            stream,
        );
    } else if buffer.starts_with(hayley_route) {
        thread::sleep(Duration::from_secs(5));
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
