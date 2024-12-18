pub mod my_socket {
    use std::net::{Ipv6Addr, SocketAddrV6};

    use socket2::{Domain, Protocol, SockAddr, Socket, Type};
    use tokio::net::TcpListener;

    use crate::error::my_errors::ErrorType;

    pub fn create_socket(port: u16) -> Result<Socket, ErrorType> {
        let socket = match Socket::new(Domain::IPV6, Type::STREAM, Some(Protocol::TCP)) {
            Ok(s) => s,
            Err(_) => {
                let error = ErrorType::SocketError(String::from("Creating socket"));
                return Err(error);
            }
        };

        match socket.set_reuse_address(true) {
            Ok(_) => (),
            Err(_) => {
                let error = ErrorType::SocketError(String::from(
                    "Problem when attempting to set reuse address",
                ));
                return Err(error);
            }
        };

        let socket_address = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), port, 0, 0);

        let socket_address = SockAddr::from(socket_address);
        match socket.bind(&socket_address) {
            Ok(_) => (),
            Err(_) => {
                let error =
                    ErrorType::SocketError(String::from("Problem when binding address to socket"));
                return Err(error);
            }
        };

        match socket.listen(128) {
            Ok(_) => (),
            Err(_) => {
                let error =
                    ErrorType::SocketError(String::from("Problem when binding address to socket"));
                return Err(error);
            }
        };

        println!("Listening on [::1]:{port}...");

        return Ok(socket);
    }

    pub fn get_listener(socket: Socket) -> Result<TcpListener, ErrorType> {
        let std_listener: std::net::TcpListener = socket.into();
        match std_listener.set_nonblocking(true) {
            Ok(s) => s,
            Err(_) => {
                return Err(ErrorType::SocketError(String::from(
                    "Problem when setting non blocking",
                )))
            }
        };

        return match TcpListener::from_std(std_listener) {
            Ok(l) => Ok(l),
            Err(_) => Err(ErrorType::SocketError(String::from(
                "Problem when converting tcp listener",
            ))),
        };
    }
}

pub mod connections {
    #![allow(dead_code, unused_variables)]

    use std::net::SocketAddr;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::sync::broadcast::Sender;
    use tokio::sync::{broadcast, Mutex, Semaphore};
    use tokio::{fs, time};

    use crate::request_validation::handle_request;
    use crate::shutdown::Message;
    use crate::{ErrorType, Logger};

    const MAX_CONNECTIONS: usize = 5;

    #[derive(Debug)]
    pub struct Listener {
        pub listener: TcpListener,
        pub connection_limit: Arc<Semaphore>,
        pub shutdown_tx: Arc<Mutex<Sender<Message>>>,
    }

    #[derive(Debug)]
    pub struct ConnectionHandler {
        pub stream: TcpStream,
        pub addr: SocketAddr,
        pub shutdown_rx: broadcast::Receiver<Message>,
    }

    pub async fn handle_connection(stream: &mut TcpStream) -> Result<(), ErrorType> {
        let mut buffer = [0; 4096];

        let bytes_read: usize = match stream.read(&mut buffer).await {
            Ok(n) => match n {
                0 => return Ok(()),
                _ => n,
            },
            Err(e) => {
                let error: ErrorType =
                    ErrorType::SocketError(String::from("Failed to read from socket"));
                return Err(error);
            }
        };

        handle_request(&buffer[..bytes_read])?;

        //println!("{:?}", String::from_utf8(buffer[..bytes_read].to_vec()));

        if buffer.starts_with(get_route("Home")) {
            format_response(
                "HTTP/1.1 200 OK",
                fs::read_to_string("html/index.html").await.unwrap(),
                stream,
            )
            .await;
        } else if buffer.starts_with(get_route("hayley")) {
            thread::sleep(Duration::from_secs(5));
            format_response(
                "HTTP/1.1 200 OK",
                fs::read_to_string("html/index.html").await.unwrap(),
                stream,
            )
            .await;
        } else {
            format_response(
                "HTTP/1.1 200 OK",
                fs::read_to_string("html/index.html").await.unwrap(),
                stream,
            )
            .await;
        }
        return Ok(());
    }

    pub async fn format_response(status_line: &str, contents: String, stream: &mut TcpStream) {
        let length: usize = contents.len();
        let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
        stream.write_all(response.as_bytes()).await.unwrap();
    }

    pub fn get_route(route: &str) -> &'static [u8] {
        return match route {
            "Home" => b"GET / HTTP/1.1",
            "hayley" => b"GET /hayley HTTP/1.1",
            _ => b"GET / HTTP/1.1",
        };
    }

    pub fn validate_request(req: &[u8]) -> Result<(), ErrorType> {
        return Ok(());
    }

    impl Listener {
        pub async fn run(&mut self, logger: Arc<Mutex<Logger>>) -> Result<(), ErrorType> {
            loop {
                let logger = Arc::clone(&logger);
                // Returns an error when the semaphore has been closed, since I do not close it
                // unwrap should be safe
                let permit = self.connection_limit.clone().acquire_owned().await.unwrap();

                let (stream, addr) = self.accept().await?;
                let mut handler = ConnectionHandler {
                    stream,
                    addr,
                    shutdown_rx: self.shutdown_tx.lock().await.subscribe(),
                };

                self.shutdown_tx
                    .lock()
                    .await
                    .send(Message::ServerRunning)
                    .unwrap();

                println!("Permit aquired for :{:?}", permit);

                tokio::spawn(async move {
                    match handler.run().await {
                        Ok(_) => (),
                        Err(e) => {
                            logger.lock().await.log_error(&e);
                        }
                    };
                    println!("Permit dropped for :{:?}", permit);
                    drop(permit);
                });
            }
        }

        pub async fn accept(&mut self) -> Result<(TcpStream, SocketAddr), ErrorType> {
            let mut backoff: usize = 200;

            loop {
                // If socket it accepted then return the associated handler
                match self.listener.accept().await {
                    Ok((stream, addr)) => {
                        return Ok((stream, addr));
                    }
                    Err(_) => {
                        // Attempt has failed too many times
                        if backoff > 6000 {
                            return Err(ErrorType::SocketError(String::from(
                                "Error establishing connection",
                            )));
                        }
                    }
                }

                // Exponential backoff to reduce contention
                time::sleep(Duration::from_millis(backoff as u64)).await;
                backoff *= 2;
            }
        }
    }

    impl ConnectionHandler {
        pub async fn run(&mut self) -> Result<(), ErrorType> {
            let msg: Message = match self.shutdown_rx.recv().await {
                Ok(m) => m,
                Err(_) => {
                    return Err(ErrorType::ConnectionError(String::from(
                        "Unable to receive message from shutdown sender",
                    )))
                }
            };

            while msg != Message::Terminate {
                handle_connection(&mut self.stream).await?;
                if !self.shutdown_rx.is_empty() {
                    let msg: Message = match self.shutdown_rx.recv().await {
                        Ok(m) => m,
                        Err(_) => {
                            return Err(ErrorType::ConnectionError(String::from(
                                "Unable to receive message from shutdown sender",
                            )))
                        }
                    };
                }
            }
            return Ok(());
        }
    }
}
