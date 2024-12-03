use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

//#![allow(unused_imports)]
use anyhow::Error;
use redis_starter_rust::db::Data;
use redis_starter_rust::redis_parser::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{channel, Receiver};
use tokio::sync::Mutex;

const PORT: &str = "127.0.0.1:6379";
#[tokio::main]
//#[allow(unreachable_code)]

async fn main() -> Result<(), Error> {
    let data: Arc<Mutex<Data>> = Arc::new(Mutex::new(Data::new()));
    let (tx, mut rx): (Sender<Message>, Receiver<Message>) = channel(10);

    let tx = Arc::new(Mutex::new(tx));

    let listener: TcpListener = TcpListener::bind(PORT).await?;
    println!("Listening on 127.0.0.1:6379");

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            thread::sleep(Duration::from_secs(msg.time));
            Arc::clone(&data).lock().await.remove(msg.key);
        }
    });

    loop {
        let (client, _addr) = listener.accept().await?;
        handle_connection(client, Arc::clone(&data), Arc::clone(&tx))?;
    }
}

fn handle_connection(
    mut client: TcpStream,
    data: Arc<Mutex<Data>>,
    tx: Arc<Mutex<Sender<Message>>>,
) -> Result<(), Error> {
    tokio::spawn(async move {
        let mut buffer: [u8; 1024] = [0; 1024];

        loop {
            // read the bytes into the buffer
            let bytes_read = client
                .read(&mut buffer)
                .await
                .expect("Failed to read data from client");

            // if no bytes were read the conntection is closed
            if bytes_read <= 0 {
                return ();
            }

            // Convert request into a string
            let request: String = String::from_utf8(buffer.to_vec()).unwrap();

            // Get the response based on the redis type
            let mut response: RedisType = get_redis_response(msg, Arc::clone(&data)).await;

            // if the response requires delayed operation
            if response.is_delay() {
                let my_tx: Arc<Mutex<Sender<Message>>> = Arc::clone(&tx);
                match &msg {
                    Command::DELAY(message) => {
                        let res: RedisType = RedisType::SimpleString(String::from("OK"));
                        client
                            .write(res.to_string().as_bytes())
                            .await
                            .expect("Failed to write to client");

                        my_tx.lock().await.send(message.clone());
                    }
                    _ => (),
                }
                return ();
            }

            client
                .write(response.to_string().as_bytes())
                .await
                .expect("Failed to write to client");
        }
    });

    return Ok(());
}
