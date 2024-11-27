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

    use tokio::net::{TcpListener, TcpStream};
    use tokio::sync::{broadcast, Semaphore};

    use crate::shutdown::Message;
    use crate::Shutdown;

    const MAX_CONNECTIONS: usize = 5;

    #[derive(Debug)]
    pub struct Listener {
        pub listener: TcpListener,
        pub connection_limit: Arc<Semaphore>,
        pub shutdown_tx: Shutdown,
    }

    #[derive(Debug)]
    pub struct ConnectionHandler {
        pub stream: TcpStream,
        pub addr: SocketAddr,
        pub shutdown_rx: broadcast::Receiver<Message>,
    }
}
