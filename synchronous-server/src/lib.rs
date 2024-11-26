pub mod my_errors {
    use std::fmt;

    #[derive(Debug, Clone)]
    pub struct SocketError {
        pub msg: String,
    }

    impl fmt::Display for SocketError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Error with socket: {}", self.msg)
        }
    }
}
