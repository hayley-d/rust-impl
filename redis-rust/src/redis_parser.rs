use std::fmt::Display;
use std::str::Chars;

pub enum RedisType {
    SimpleString(String),
    Error(String),
    Integer(String),
    BulkString(String),
    Array(Box<[RedisType]>),
    Null,
    Boolean(bool),
}

pub fn get_redis_command(req: String) -> Command {
    if req.to_uppercase().contains("ECHO") {
        let mut msg: Vec<&str> = req.split("\r\n").collect();
        msg.pop();
        let index: usize = msg
            .iter()
            .position(|&s| s.to_uppercase() == "ECHO")
            .unwrap()
            + 1;
        let mut req_msg = String::new();
        for s in index..msg.len() {
            let symbols: Vec<char> = vec!['*', ':', '+', '-', '$', '_', '#'];
            if !msg[s].contains(&symbols[..]) {
                req_msg.push_str(msg[s]);
            }
        }

        return Command::ECHO(req_msg);
    } else {
        //PING
        return Command::PING(String::new());
    }
}

impl Display for RedisType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            RedisType::SimpleString(msg) => {
                return write!(f, "+{}\r\n", msg);
            }
            RedisType::Error(msg) => {
                return write!(f, "-{}\r\n", msg);
            }
            RedisType::Integer(msg) => {
                return write!(f, ":{}\r\n", msg);
            }
            RedisType::BulkString(msg) => {
                return write!(f, "${}\r\n{}\r\n", &msg.len().to_string(), msg);
            }
            RedisType::Array(elements) => {
                let mut res: String = String::new();
                for element in elements {
                    res.push_str(&element.to_string());
                }
                return write!(f, "*{}\r\n{}", &elements.len().to_string(), res);
            }
            RedisType::Null => {
                return write!(f, "_\r\n");
            }
            RedisType::Boolean(msg) => match msg {
                true => return write!(f, "#t\r\n"),
                false => return write!(f, "#f\r\n"),
            },
        }
    }
}

pub fn split_command(command: &str) -> Option<Vec<&str>> {
    let iter = command.char_indices();

    let mut start: isize = -1;

    for (pos, c) in iter {
        if c.is_whitespace() {
            if start == -1 {
                start = pos as isize;
                break;
            }
        }
    }
    if start == -1 {
        return None;
    }

    let start: usize = start as usize;
    return Some(vec![&command[..start], &command[start + 1..]]);
}

#[derive(Debug)]
pub enum Command {
    PING(String),
    ECHO(String),
    ERROR(String),
}

impl Command {
    pub fn new(command: &str, content: String) -> Command {
        match command.to_uppercase().as_str() {
            "ECHO" => Command::ECHO(content),
            "PING" => Command::PING(content),
            _ => Command::ERROR(content),
        }
    }

    pub fn get_response(&mut self) -> RedisType {
        match &self {
            Command::ECHO(msg) => RedisType::BulkString(msg.to_string()),
            Command::PING(_) => RedisType::SimpleString(String::from("PONG")),
            Command::ERROR(msg) => RedisType::Error(msg.to_string()),
        }
    }
}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        match &self {
            Command::ECHO(_) => match other {
                Command::ECHO(_) => true,
                _ => false,
            },
            Command::PING(_) => match other {
                Command::PING(_) => true,
                _ => false,
            },
            Command::ERROR(_) => match other {
                Command::ERROR(_) => true,
                _ => false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_test() {
        let sen1: String = String::from("ECHO Hello world");
        let res1 = split_command(&sen1).unwrap();
        assert_eq!(res1, vec!["ECHO", "Hello world"]);
    }

    #[test]
    fn test_req_split() {
        let msg: &str = "*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n";
        let mut msg: Vec<&str> = msg.split("\r\n").collect();
        msg.pop();
        assert_eq!(msg, vec!["*2", "$4", "ECHO", "$3", "hey"]);
    }

    #[test]
    fn get_command_test() {
        let msg: String = String::from("*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n");

        assert_eq!(get_redis_command(msg), Command::ECHO(String::from("hey")));
    }

    #[test]
    fn command_convert_test() {
        let sen1: String = String::from("ECHO Hello world");
        let res1: Vec<&str> = split_command(&sen1).unwrap();

        assert_eq!(
            Command::new(res1[0], res1[1].to_string()),
            Command::ECHO("Hello world".to_string())
        );
    }

    #[test]
    fn simple_string_test() {
        let my_simple: RedisType = RedisType::SimpleString(String::from("OK"));
        assert_eq!(my_simple.to_string(), "+OK\r\n");
    }

    #[test]
    fn error_test() {
        let my_error: RedisType = RedisType::Error(String::from("Err unknown command"));
        assert_eq!(my_error.to_string(), "-Err unknown command\r\n");
    }

    #[test]
    fn integer_test() {
        let my_int: RedisType = RedisType::Integer(String::from("0"));
        assert_eq!(my_int.to_string(), ":0\r\n");
    }

    #[test]
    fn bulk_test() {
        let my_bulk: RedisType = RedisType::BulkString(String::from("Hello world"));
        assert_eq!(my_bulk.to_string(), "$11\r\nHello world\r\n");
    }

    #[test]
    fn array_test_string() {
        let array: Box<[RedisType]> = Box::new([
            RedisType::BulkString(String::from("Hello")),
            RedisType::BulkString(String::from("world")),
        ]);

        let my_array_type: RedisType = RedisType::Array(array);
        assert_eq!(
            my_array_type.to_string(),
            "*2\r\n$5\r\nHello\r\n$5\r\nworld\r\n"
        );
    }

    #[test]
    fn array_test_int() {
        let array: Box<[RedisType]> = Box::new([
            RedisType::Integer(String::from("1")),
            RedisType::Integer(String::from("2")),
            RedisType::Integer(String::from("3")),
        ]);

        let my_array_type: RedisType = RedisType::Array(array);
        assert_eq!(my_array_type.to_string(), "*3\r\n:1\r\n:2\r\n:3\r\n");
    }

    #[test]
    fn null_test() {
        let my_null: RedisType = RedisType::Null;
        assert_eq!(my_null.to_string(), "_\r\n");
    }

    #[test]
    fn bool_test() {
        let my_truthy: RedisType = RedisType::Boolean(true);
        let my_falsy: RedisType = RedisType::Boolean(false);
        assert_eq!(my_truthy.to_string(), "#t\r\n");
        assert_eq!(my_falsy.to_string(), "#f\r\n");
    }
}
