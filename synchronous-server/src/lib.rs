pub mod my_errors {
    use std::fmt;

    #[derive(Debug, Clone)]
    pub struct SocketError {
        pub msg: String,
    }

    impl fmt::Display for SocketError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Error with socket: {}", self.msg)
        }
    }
}

pub mod my_socket {
    use std::net::{Ipv6Addr, SocketAddrV6};

    use socket2::{Domain, Protocol, SockAddr, Socket, Type};

    use crate::my_errors::SocketError;

    pub fn create_socket() -> Result<Socket, SocketError> {
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
}
