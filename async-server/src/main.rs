use async_server::{my_errors::SocketError, my_socket::create_socket, my_threadpool::ThreadPool};
use std::net::SocketAddr;
use std::sync::Arc;
use std::{fs, io::prelude::*, thread, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), SocketError> {
    let socket = create_socket()?;

    // Convert std listener into tokio listener
    let std_listener: std::net::TcpListener = socket.into();
    match std_listener.set_nonblocking(true) {
        Ok(s) => s,
        Err(_) => {
            let error = SocketError {
                msg: String::from("Error creating socket"),
            };
            eprintln!("{}", error);
            return Err(error);
        }
    };

    let listener = match TcpListener::from_std(std_listener) {
        Ok(l) => l,
        Err(_) => {
            let error = SocketError {
                msg: String::from("Error creating socket"),
            };
            eprintln!("{}", error);
            return Err(error);
        }
    };

    let thread_pool: Arc<ThreadPool> = Arc::new(ThreadPool::new(5)?);
    let mut count: usize = 0;
    loop {
        if count < 2 {
            break;
        }
        count += 1;

        let (mut stream, mut add): (TcpStream, SocketAddr) = match listener.accept().await {
            Ok((s, add)) => (s, add),
            Err(_) => {
                let error = SocketError {
                    msg: String::from("Error creating socket"),
                };
                eprintln!("{}", error);
                return Err(error);
            }
        };

        let my_pool = Arc::clone(&thread_pool);
        tokio::spawn(async move {
            my_pool.execute(async || {
                handle_connection(stream).await;
            });
        });
    }

    Ok(())
}

async fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer).await {
        Ok(n) if n == 0 => return,
        Ok(n) => n,
        Err(e) => {
            eprintln!("Failed to read from socket: {:?}", e);
            return;
        }
    };

    let home_route: &[u8] = b"GET / HTTP/1.1";
    let hayley_route: &[u8] = b"GET /hayley HTTP/1.1";

    if buffer.starts_with(home_route) {
        format_response(
            "HTTP/1.1 200 OK",
            fs::read_to_string("html/index.html").unwrap(),
            stream,
        )
        .await;
    } else if buffer.starts_with(hayley_route) {
        thread::sleep(Duration::from_secs(5));
    } else {
        format_response(
            "HTTP/1.1 200 OK",
            fs::read_to_string("html/hello.html").unwrap(),
            stream,
        )
        .await;
    }
}

async fn format_response(status_line: &str, contents: String, mut stream: TcpStream) {
    let length: usize = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    match stream.write_all(response.as_bytes()).await {
        Ok(_) => (),
        Err(_) => (),
    };
}
