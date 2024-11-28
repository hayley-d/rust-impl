use self::my_socket::{create_socket, get_listener};
use async_server::connection::*;
use async_server::error::my_errors::*;
use socket2::{Socket, Type};

#[test]
fn test_connection() {
    let logger: Logger = Logger::new("server.log");

    let socket: Socket = match create_socket(7878 as u16) {
        Ok(s) => s,
        Err(e) => {
            logger.log_error(&e);
            panic!("Error creating socket, refer to the server log");
        }
    };

    assert_eq!(Socket::type(&socket).unwrap(),Type::STREAM);

    let listener = match get_listener(socket) {
        Ok(s) => s,
        Err(e) => {
            logger.log_error(&e);
            panic!("Error creating listener, refer to the server log");
        }
    };
}
