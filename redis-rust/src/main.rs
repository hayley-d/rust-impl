use std::sync::Arc;
use std::thread;
use std::time::Duration;

//#![allow(unused_imports)]
use anyhow::Error;
use redis_starter_rust::db::Database;
use redis_starter_rust::redis_parser::*;
use std::env;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;

const PORT: &str = "127.0.0.1:6379";
#[tokio::main]
//#[allow(unreachable_code)]

async fn main() -> Result<(), Error> {
    // get the dir and file name from env
    let args: Vec<String> = env::args().collect();

    let data: Arc<Mutex<Database>> = Arc::new(Mutex::new(Database::new()));
    // add args to the map
    if args.len() == 5 {
        println!("Adding env args to the hash map");
        data.lock()
            .await
            .add("dir".into(), args.get(2).unwrap().into());

        data.lock()
            .await
            .add("dbfilename".into(), args.get(4).unwrap().into());
    }

    let (tx, mut rx): (Sender<Message>, Receiver<Message>) = channel(10);

    let tx = Arc::new(Mutex::new(tx));

    let listener: TcpListener = TcpListener::bind(PORT).await?;
    println!("Listening on {PORT}");

    let data_copy = Arc::clone(&data);
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            thread::sleep(Duration::from_millis(msg.time));
            data_copy.lock().await.remove(msg.key);
            println!("Key value pair has been removed");
        }
    });

    loop {
        let (client, _addr) = listener.accept().await?;
        handle_connection(client, Arc::clone(&data), Arc::clone(&tx))?;
    }
}

fn handle_connection(
    mut client: TcpStream,
    data: Arc<Mutex<Database>>,
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

            println!("Request: {}", request);

            // Get the response based on the redis type
            let response: RedisType = get_redis_response(request, Arc::clone(&data)).await;

            // if the response requires delayed operation
            if response.is_delay() {
                match &response {
                    RedisType::Delay(message) => {
                        let res: RedisType = RedisType::SimpleString(String::from("OK"));
                        client
                            .write(res.to_string().as_bytes())
                            .await
                            .expect("Failed to write to client");

                        let _ = Arc::clone(&tx).lock().await.send(message.clone()).await;
                        continue;
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
