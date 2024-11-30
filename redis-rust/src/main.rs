#![allow(unused_imports)]
use std::io::{BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

use anyhow::Error;

fn main() -> Result<(), Error> {
    println!("Listening on 127.0.0.1:6379");

    let listener: TcpListener = TcpListener::bind("127.0.0.1:6379").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let mut buf = [0; 512];
                loop {
                    let read_count = stream.read(&mut buf).unwrap();
                    if read_count == 0 {
                        break;
                    }
                    stream.write(b"+PONG\r\n").unwrap();
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}
