use std::fmt::Display;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::Data;

pub enum RedisType {
    SimpleString(String),
    Error(String),
    Integer(String),
    BulkString(String),
    Array(Box<[RedisType]>),
    Null,
    Boolean(bool),
    NullBulk,
}

#[derive(Debug)]
pub struct Message {
    pub key: String,
    pub time: u64,
}

pub async fn get_redis_command(req: String, data: Arc<Mutex<Data>>) -> Command {
    let mut msg: Vec<&str> = req.split("\r\n").collect();
    msg.pop();
    let data = data;
    let _data = Arc::clone(&data);

    if req.to_uppercase().contains("ECHO") {
        let index: usize = msg
            .iter()
            .position(|&s| s.to_uppercase() == "ECHO")
            .unwrap()
            + 1;

        return Command::ECHO(extract_msg(msg, index, None)[0].to_string());
    } else if req.to_uppercase().contains("SET") {
        let index: usize = msg.iter().position(|&s| s.to_uppercase() == "SET").unwrap() + 1;
        let req_msg: Vec<String> = extract_msg(msg, index, None);
        _data
            .lock()
            .await
            .add(req_msg[0].to_string(), req_msg[1].to_string());
        if "px" == req_msg[2].to_lowercase() {
            return Command::DELAY(Message::new(
                req_msg[3].parse::<u64>().unwrap(),
                req_msg[0].clone(),
            ));
        }

        return Command::SIMPLE("OK".to_string());
    } else if req.to_uppercase().contains("GET") {
        let index: usize = msg.iter().position(|&s| s.to_uppercase() == "GET").unwrap() + 1;

        let req_msg: Vec<String> = extract_msg(msg, index, Some(1));

        match data.lock().await.get(req_msg[0].to_string()) {
            Some(d) => {
                return Command::BULK(d.clone());
            }
            None => return Command::NULLBULK,
        };
    } else {
        //PING
        return Command::PING;
    }
}

fn extract_msg(req: Vec<&str>, start: usize, count: Option<usize>) -> Vec<String> {
    let symbols: Vec<char> = vec!['*', ':', '+', '-', '$', '_', '#'];
    let mut req_msg: Vec<String> = Vec::new();
    match count {
        Some(num) => {
            let mut count = 0;
            for s in start..req.len() {
                if !req[s].contains(&symbols[..]) {
                    req_msg.push(req[s].to_string());
                    count += 1;
                    if count >= num {
                        break;
                    }
                }
            }
        }
        None => {
            for s in start..req.len() {
                if !req[s].contains(&symbols[..]) {
                    req_msg.push(req[s].to_string());
                }
            }
        }
    }
    return req_msg;
}

impl Message {
    pub fn new(time: u64, key: String) -> Self {
        return Message { key, time };
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        return self.key == other.key && self.time == other.time;
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
            RedisType::NullBulk => {
                return write!(f, "$-1\r\n");
            }
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
    PING,
    ECHO(String),
    ERROR(String),
    SIMPLE(String),
    BULK(String),
    NULLBULK,
    DELAY(Message),
}

impl Command {
    pub fn new(command: &str, content: String) -> Command {
        match command.to_uppercase().as_str() {
            "ECHO" => Command::ECHO(content),
            "PING" => Command::PING,
            "SIMPLE" => Command::SIMPLE(content),
            "BULK" => Command::BULK(content),
            _ => Command::ERROR(content),
        }
    }

    pub fn get_response(&mut self) -> RedisType {
        match &self {
            Command::ECHO(msg) => RedisType::BulkString(msg.to_string()),
            Command::PING => RedisType::SimpleString(String::from("PONG")),
            Command::ERROR(msg) => RedisType::Error(msg.to_string()),
            Command::SIMPLE(msg) => RedisType::SimpleString(msg.to_string()),
            Command::BULK(msg) => RedisType::BulkString(msg.to_string()),
            Command::NULLBULK => RedisType::NullBulk,
            Command::DELAY(_) => RedisType::SimpleString(String::from("OK")),
        }
    }

    pub fn get_msg(&self) -> Option<String> {
        match &self {
            Command::ECHO(msg) => Some(msg.clone()),
            Command::PING => None,
            Command::ERROR(msg) => Some(msg.clone()),
            Command::SIMPLE(msg) => Some(msg.clone()),
            Command::BULK(msg) => Some(msg.clone()),
            Command::NULLBULK => None,
            Command::DELAY(message) => Some(message.time.to_string()),
        }
    }

    pub fn get_key(&self) -> Option<&String> {
        match &self {
            Command::DELAY(message) => Some(&message.key),
            _ => None,
        }
    }

    pub fn is_delay(&self) -> bool {
        match &self {
            Command::DELAY(_) => true,
            _ => false,
        }
    }
}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        match &self {
            Command::ECHO(msg1) => match other {
                Command::ECHO(msg2) => {
                    if msg1 == msg2 {
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
            Command::PING => match other {
                Command::PING => true,
                _ => false,
            },
            Command::ERROR(msg1) => match other {
                Command::ERROR(msg2) => {
                    if msg1 == msg2 {
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
            Command::SIMPLE(msg1) => match other {
                Command::SIMPLE(msg2) => {
                    if msg1 == msg2 {
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
            Command::BULK(msg1) => match other {
                Command::BULK(msg2) => {
                    if msg1 == msg2 {
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
            Command::NULLBULK => match other {
                Command::NULLBULK => true,
                _ => false,
            },
            Command::DELAY(message1) => match other {
                Command::DELAY(message2) => {
                    if message1 == message2 {
                        true
                    } else {
                        false
                    }
                }
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

    #[tokio::test]
    async fn echo_command_test() {
        let data = Arc::new(Mutex::new(Data::new()));
        let msg: String = String::from("*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n");
        assert_eq!(
            get_redis_command(msg, data).await,
            Command::ECHO(String::from("hey"))
        );
    }

    #[tokio::test]
    async fn get_command_test() {
        let data = Arc::new(Mutex::new(Data::new()));
        data.lock()
            .await
            .add(String::from("foo"), String::from("bar"));

        let msg: String = String::from("*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n");

        assert_eq!(
            get_redis_command(msg, data).await,
            Command::BULK(String::from("bar"))
        );
    }

    #[tokio::test]
    async fn set_command_test() {
        let data = Arc::new(Mutex::new(Data::new()));

        let msg: String = String::from("*3\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");

        assert_eq!(
            get_redis_command(msg, Arc::clone(&data)).await,
            Command::SIMPLE(String::from("OK"))
        );

        assert_eq!(
            *(data.lock().await.get(String::from("foo")).unwrap()),
            "bar".to_string()
        );
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
