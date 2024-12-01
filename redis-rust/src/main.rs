//#![allow(unused_imports)]
use anyhow::Error;
use redis_starter_rust::redis_parser::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const PORT: &str = "127.0.0.1:6379";
#[tokio::main]
//#[allow(unreachable_code)]

async fn main() -> Result<(), Error> {
    let listener: TcpListener = TcpListener::bind(PORT).await?;
    println!("Listening on 127.0.0.1:6379");

    loop {
        let (client, _addr) = listener.accept().await?;
        handle_connection(client)?;
    }
}

fn handle_connection(mut client: TcpStream) -> Result<(), Error> {
    tokio::spawn(async move {
        let mut buffer: [u8; 1024] = [0; 1024];

        loop {
            let bytes_read = client
                .read(&mut buffer)
                .await
                .expect("Failed to read data from client");

            if bytes_read <= 0 {
                return ();
            }

            let msg: String = String::from_utf8(buffer.to_vec()).unwrap();

            //let parts: Vec<&str> = split_command(msg).expect("Error parsing command");

            //let mut command: Command = Command::new(parts[0], parts[1].to_string());

            //let response: RedisType = command.get_response();

            /*client
                            .write(response.to_string().as_bytes())
                            .await
                            .expect("Failed to write to client");
            */
        }
    });

    return Ok(());
}
