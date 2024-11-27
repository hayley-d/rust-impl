pub mod my_errors {
    use std::fmt;
    use std::io::Write;
    use std::sync::Arc;

    pub enum ErrorType {
        SocketError(String),
        ReadError(String),
        WriteError(String),
        BadRequest(String),
        NotFound(String),
        InternalServerError(String),
        ProtocolError(String),
        ConnectionError(String),
    }

    #[derive(Debug, Clone)]
    pub struct SocketError {
        pub msg: String,
    }

    pub struct Logger {
        log_file: Arc<std::sync::Mutex<std::fs::File>>,
    }

    impl fmt::Display for ErrorType {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                ErrorType::SocketError(msg) => write!(f, "Error with socket: {}", msg),
                ErrorType::ReadError(msg) => write!(f, "Error reading file: {}", msg),
                ErrorType::WriteError(msg) => write!(f, "Error writing to file: {}", msg),
                ErrorType::BadRequest(msg) => write!(f, "Error bad request: {}", msg),
                ErrorType::NotFound(msg) => write!(f, "Error resource not found: {}", msg),
                ErrorType::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
                ErrorType::ProtocolError(msg) => write!(f, "Protocol Error: {}", msg),
                ErrorType::ConnectionError(msg) => write!(f, "Connection Error: {}", msg),
            }
        }
    }

    impl fmt::Debug for ErrorType {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                ErrorType::SocketError(msg) => {
                    write!(
                        f,
                        "Socket Error: {{ file: {}, line: {} message: {} }}",
                        file!(),
                        line!(),
                        msg
                    )
                }
                ErrorType::ReadError(msg) => {
                    write!(
                        f,
                        "Read Error: {{ file: {}, line: {} message: {} }}",
                        file!(),
                        line!(),
                        msg
                    )
                }
                ErrorType::WriteError(msg) => {
                    write!(
                        f,
                        "Write Error: {{ file: {}, line: {} message: {} }}",
                        file!(),
                        line!(),
                        msg
                    )
                }
                ErrorType::BadRequest(msg) => {
                    write!(
                        f,
                        "Bad Request Error: {{ file: {}, line: {} message: {} }}",
                        file!(),
                        line!(),
                        msg
                    )
                }

                ErrorType::NotFound(msg) => {
                    write!(
                        f,
                        "Resource Not Found Error: {{ file: {}, line: {} message: {} }}",
                        file!(),
                        line!(),
                        msg
                    )
                }
                ErrorType::InternalServerError(msg) => {
                    write!(
                        f,
                        "Internal Server Error: {{ file: {}, line: {} message: {} }}",
                        file!(),
                        line!(),
                        msg
                    )
                }
                ErrorType::ProtocolError(msg) => {
                    write!(
                        f,
                        "Protocol Error: {{ file: {}, line: {} message: {} }}",
                        file!(),
                        line!(),
                        msg
                    )
                }
                ErrorType::ConnectionError(msg) => write!(
                    f,
                    "Connection Error: {{ file: {}, line: {} message: {} }}",
                    file!(),
                    line!(),
                    msg
                ),
            }
        }
    }

    impl Logger {
        pub fn new(log_path: &str) -> Self {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
                .expect("Failed to open log file");
            return Logger {
                log_file: Arc::new(std::sync::Mutex::new(file)),
            };
        }

        pub fn log_error(&self, error: &ErrorType) {
            let mut file = self.log_file.lock().unwrap();
            let log_message = format!("[{}] {}\n", chrono::Utc::now(), error);
            file.write_all(log_message.as_bytes())
                .expect("Failed to write to log file");
        }
    }
}

pub mod my_socket {
    use std::net::{Ipv6Addr, SocketAddrV6};

    use socket2::{Domain, Protocol, SockAddr, Socket, Type};

    use crate::my_errors::ErrorType;

    pub fn create_socket() -> Result<Socket, ErrorType> {
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

        let socket_address = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 7878, 0, 0);

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

        println!("Listening on [::1]:7878...");

        return Ok(socket);
    }
}

// Not needed since tokio runtime will be used
/*pub mod my_threadpool {
    use std::sync::mpsc::{self, Receiver, Sender};
    use std::sync::{Arc, Mutex};
    use std::thread::{self, JoinHandle};

    use tokio::runtime::Handle;

    use crate::my_errors::SocketError;

    pub enum Message {
        NewJob(Job),
        Terminate,
    }

    pub struct ThreadPool {
        pub workers: Vec<Worker>,
        pub capacity: usize,
        tx: Sender<Message>,
    }

    pub struct Worker {
        id: usize,
        thread: Option<JoinHandle<()>>,
    }

    type Job = Box<dyn FnOnce() + Send + 'static>;

    impl ThreadPool {
        pub fn new(size: usize) -> Result<ThreadPool, SocketError> {
            if size <= 0 {
                return Err(SocketError {
                    msg: String::from("Invalid thread pool size"),
                });
            }

            let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel();

            let rx = Arc::new(Mutex::new(rx));
            let mut workers: Vec<Worker> = Vec::with_capacity(size);
            for idx in 0..size {
                workers.push(Worker::new(idx, Arc::clone(&rx)));
            }

            return Ok(ThreadPool {
                workers,
                capacity: size,
                tx,
            });
        }

        pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static,
        {
            let job = Box::new(f);
            self.tx.send(Message::NewJob(job)).unwrap();
        }

        pub fn execute_async<F>(&self, f: F)
        where
            F: std::future::Future<Output = ()> + Send + 'static,
        {
            let handle = Handle::current();
            self.execute(move || {
                handle.block_on(f);
            });
        }
    }

    impl Worker {
        pub fn new(id: usize, rx: Arc<Mutex<Receiver<Message>>>) -> Worker {
            return Worker {
                id,
                thread: Some(thread::spawn(move || loop {
                    let job = match rx.lock().unwrap().recv() {
                        Ok(Message::NewJob(j)) => j,
                        Ok(Message::Terminate) => break,
                        Err(_) => continue,
                    };
                    job();
                })),
            };
        }
    }

    impl Drop for ThreadPool {
        fn drop(&mut self) {
            for _ in &self.workers {
                let _ = self.tx.send(Message::Terminate);
            }

            for worker in &mut self.workers {
                if let Some(thread) = worker.thread.take() {
                    thread.join().unwrap();
                }
            }
        }
    }
}*/
