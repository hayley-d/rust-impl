use codecrafters_http_server::error::*;
use codecrafters_http_server::response::{Code, Response};
#[allow(unused_imports)]
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    let listener: TcpListener = match TcpListener::bind("127.0.0.1:4221").await {
        Ok(s) => s,
        Err(_) => {
            return Err(ServerError {
                message: String::from("binding to port"),
            });
        }
    };

    loop {
        let (mut client, addr) = match listener.accept().await {
            Ok((s, a)) => {
                println!("Accepted connection at {}", a);
                (s, a)
            }
            Err(_) => {
                return Err(ServerError {
                    message: String::from("accepting client"),
                })
            }
        };

        tokio::spawn(async move {
            let mut buffer: [u8; 4096] = [0; 4096];

            loop {
                let n = match client.read(&mut buffer).await {
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(_) => {
                        eprintln!(
                            "{}",
                            ServerError {
                                message: String::from("reading client request"),
                            }
                        );
                        return;
                    }
                };

                let response = Response {
                    message: String::from("Ok"),
                    code: Code::Ok,
                };

                let response = response.to_string();

                if let Err(e) = client.write(response.as_bytes()).await {
                    eprintln!("Error: {:?}", e);
                    return;
                }
            }
        });
    }
}
