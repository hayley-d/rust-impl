use async_server::my_errors::Logger;
use async_server::{my_errors::ErrorType, my_socket::create_socket};
use std::{fs, thread, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), ErrorType> {
    let logger: Logger = Logger::new("server.log");

    let socket = match create_socket() {
        Ok(s) => s,
        Err(e) => {
            logger.log_error(&e);
            panic!("Error creating socker, refer to the server log");
        }
    };

    // Convert std listener into tokio listener
    let std_listener: std::net::TcpListener = socket.into();
    match std_listener.set_nonblocking(true) {
        Ok(s) => s,
        Err(_) => {
            let error = ErrorType::SocketError(String::from("Problem when setting non blocking"));
            logger.log_error(&error);
        }
    };

    let listener = match TcpListener::from_std(std_listener) {
        Ok(l) => l,
        Err(_) => {
            let error =
                ErrorType::SocketError(String::from("Problem when converting tcp listener"));
            logger.log_error(&error);
            panic!("Error converting std::TcpListener to tokio::TcpListener");
        }
    };

    // Graceful shutdown using signal handling
    let shutdown_signal = tokio::signal::ctrl_c();

    tokio::select! {
        _ = run_server(listener,&logger) => {
            println!("Server has stopped.");
        }
        _ = shutdown_signal => {
            println!("Shutdown signal received. Stopping server...");
        }
    }

    Ok(())
}

async fn run_server(listener: TcpListener, logger: &Logger) -> Result<(), ErrorType> {
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                tokio::spawn(async move {
                    println!("Handling connection from {:?}", addr);
                    handle_connection(stream).await;
                });
            }
            Err(_) => {
                let error = ErrorType::ConnectionError(String::from("Failed to accept connection"));
                logger.log_error(&error);
                continue;
            }
        }
    }
}

async fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer).await {
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
    stream.write_all(response.as_bytes()).await.unwrap();
}
