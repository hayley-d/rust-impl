use async_server::connection::{connections::*, my_socket::*};
use async_server::error::my_errors::*;
use async_server::shutdown::*;
use std::env;
use std::sync::Arc;
use std::{fs, thread, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::Sender;
use tokio::sync::{broadcast, Mutex};

const DEFAULT_PORT: u16 = 7878;

#[tokio::main]
async fn main() -> Result<(), ErrorType> {
    let logger: Logger = Logger::new("server.log");

    let port: u16 = match env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_PORT.to_string())
        .parse()
    {
        Ok(p) => p,
        Err(_) => {
            let error = ErrorType::SocketError(String::from("Problem parsing port"));
            logger.log_error(&error);
            DEFAULT_PORT
        }
    };

    let socket = match create_socket(port) {
        Ok(s) => s,
        Err(e) => {
            logger.log_error(&e);
            panic!("Error creating socket, refer to the server log");
        }
    };

    let listener = match get_listener(socket) {
        Ok(s) => s,
        Err(e) => {
            logger.log_error(&e);
            panic!("Error creating listener, refer to the server log");
        }
    };

    let (tx, _rx) = broadcast::channel(10);
    let tx = Arc::new(Mutex::new(tx));

    let mut shutdown = Shutdown::new(Arc::clone(&tx));

    // Graceful shutdown using signal handling
    let shutdown_signal = tokio::signal::ctrl_c();

    tokio::select! {
        _ = run_server(listener,&logger,Arc::clone(&tx)) => {
            println!("Server has stopped.");
        }
        _ = shutdown_signal => {
            println!("Shutdown signal received. Stopping server...");
            shutdown.initiate_shutdown().await;
        }
    }

    Ok(())
}

async fn run_server(
    listener: TcpListener,
    logger: &Logger,
    tx: Arc<Mutex<Sender<Message>>>,
) -> Result<(), ErrorType> {
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let mut connection = ConnectionHandler {
                    stream,
                    addr,
                    shutdown_rx: tx.lock().await.subscribe(),
                };

                tokio::spawn(async move {
                    println!("Handling connection from {:?}", connection.addr);
                    tokio::select! {
                        _ = handle_connection(connection.stream)=> {
                            println!("Connection closed");
                        }
                        _ = connection.shutdown_rx.recv() => {
                            println!("Thread shutting down");
                        }
                    };
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
    let mut buffer = [0; 4096];
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
        thread::sleep(Duration::from_secs(15));
        format_response(
            "HTTP/1.1 200 OK",
            fs::read_to_string("html/index.html").unwrap(),
            stream,
        )
        .await;
    } else {
        format_response(
            "HTTP/1.1 200 OK",
            fs::read_to_string("html/index.html").unwrap(),
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
