use std::net::{Ipv6Addr, SocketAddrV6};

#[test]
fn test_socket() {
    let socket_address = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 7878, 0, 0);
    assert_eq!("[::1]:7878".parse(), Ok(socket_address));
    assert_eq!(socket_address.ip(), &Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
    assert_eq!(socket_address.port(), 7878);
}
