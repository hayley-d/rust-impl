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
