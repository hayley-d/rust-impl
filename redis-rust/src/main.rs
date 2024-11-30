#![allow(unused_imports)]
use anyhow::Error;
use std::io::{BufReader, Read, Write};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const PORT: &str = "127.0.0.1:6379";
#[tokio::main]
#[allow(unreachable_code)]

async fn main() -> Result<(), Error> {
    let listener: TcpListener = TcpListener::bind(PORT).await?;
    println!("Listening on 127.0.0.1:6379");

    loop {
        let (mut client, _addr) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buffer: [u8; 1024] = [0; 1024];

            loop {
                let bytes_read = client
                    .read(&mut buffer)
                    .await
                    .expect("Failed to read data from client");

                if bytes_read <= 0 {
                    return;
                }

                client
                    .write(b"+PONG\r\n")
                    .await
                    .expect("Failed to write to client");
            }
        });
    }
    Ok(())
}
