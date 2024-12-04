use std::fmt::Display;
use std::sync::Arc;
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;

use crate::{Database, Error};
#[derive(Debug)]
pub enum RedisType<'a> {
    SimpleString(&'a str),
    Error(&'a str),
    Integer(String),
    BulkString(String),
    Array(Box<Vec<RedisType<'a>>>),
    Null,
    Boolean(bool),
    NullBulk,
    Delay(Message<'a>),
}

#[derive(Debug)]
pub struct Message<'a> {
    pub key: &'a str,
    pub time: u64,
}
#[allow(unreachable_code)]
pub async fn get_redis_response<'a>(
    req: &'a String,
    data: Arc<Mutex<Database>>,
) -> Result<RedisType<'a>, Error> {
    // Transform request into a vector
    let mut request: Vec<&str> = req.split("\r\n").collect();
    request.pop();

    // clone the arc
    let _data = Arc::clone(&data);

    if req.to_uppercase().contains("ECHO") {
        // Safe to unwrap since we know it contains ECHO
        let index: usize = request
            .iter()
            .position(|&s| s.to_uppercase() == "ECHO")
            .unwrap()
            + 1;
        return Ok(RedisType::BulkString(
            extract_msg(&request, index, None)[0].to_string(),
        ));
    } else if req.to_uppercase().contains("SET") {
        // safe to unwrap since we know it contains SET
        let index: usize = request
            .iter()
            .position(|&s| s.to_uppercase() == "SET")
            .unwrap()
            + 1;

        // extract the rest of the request message
        let request_params: Vec<&str> = extract_msg(&request, index, None);

        _data.lock().await.add(request_params[0], request_params[1]);

        if request_params.len() > 2 && "px" == request_params[2].to_lowercase() {
            // safe to unwrap here
            return Ok(RedisType::Delay(Message::new(
                request_params[3].parse::<u64>().unwrap(),
                request_params[0],
            )));
        }

        return Ok(RedisType::SimpleString("OK"));
    } else if req.to_uppercase().contains("CONFIG") {
        let mut index: usize = match request.iter().position(|&s| s.to_uppercase() == "GET") {
            Some(i) => i,
            None => {
                return Err(Error {
                    message: "Error attaing index for CONFIG command".to_string(),
                })
            }
        };

        index += 1;

        let request_params: Vec<&str> = extract_msg(&request, index, Some(1));

        match data.lock().await.get(&request_params[0].to_lowercase()) {
            Some(d) => {
                let res: Vec<RedisType> = vec![
                    RedisType::BulkString(request_params[0].to_string()),
                    RedisType::BulkString(d),
                ];
                return Ok(RedisType::Array(Box::new(res)));
            }
            None => return Ok(RedisType::NullBulk),
        };
    } else if req.to_uppercase().contains("GET") {
        let index: usize = request
            .iter()
            .position(|&s| s.to_uppercase() == "GET")
            .unwrap()
            + 1;

        let req_msg: Vec<&str> = extract_msg(&request, index, Some(1));

        match data.lock().await.get(req_msg[0]) {
            Some(d) => {
                return Ok(RedisType::BulkString(d));
            }
            None => return Ok(RedisType::NullBulk),
        };
    } else if req.to_uppercase().contains("KEYS") {
        let index: usize = request
            .iter()
            .position(|&s| s.to_uppercase() == "KEYS")
            .unwrap()
            + 1;

        let mut path: String = data.lock().await.get("dir").unwrap().to_string();

        path.push_str("/");
        path.push_str(&data.lock().await.get("dir").unwrap());

        let regex_expression: &str = extract_regex(&request, index)[1].clone();

        let mut keys: Vec<RedisType> = Vec::new();

        let contents: String = match fs::read_to_string(path).await {
            Ok(file) => file,
            Err(_) => return Ok(RedisType::Error("Error finding file")),
        };
        let mut my_file: File = match File::create("input.txt").await {
            Ok(file) => file,
            Err(_) => return Ok(RedisType::Error("Error creating file")),
        };

        if contents.contains("FC") {
            println!("Found fc");
        }

        println!("File contents: {}", contents);

        let index: usize = request
            .iter()
            .position(|&s| s.to_uppercase() == "KEYS")
            .unwrap()
            + 1;

        for key in data.lock().await.get_keys() {
            keys.push(RedisType::BulkString(key));
        }

        return Ok(RedisType::Array(Box::new(keys)));

        // get the path to the db file
        return Ok(RedisType::NullBulk);
    } else {
        //PING
        return Ok(RedisType::SimpleString("PONG"));
    }
}

fn extract_msg<'a>(req: &Vec<&'a str>, start: usize, count: Option<usize>) -> Vec<&'a str> {
    let symbols: Vec<char> = vec!['*', ':', '+', '-', '$', '_', '#'];
    let mut req_msg: Vec<&str> = Vec::new();
    match count {
        Some(num) => {
            let mut count = 0;
            for s in start..req.len() {
                if !req[s].contains(&symbols[..]) {
                    req_msg.push(req[s]);
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
                    req_msg.push(req[s]);
                }
            }
        }
    }
    return req_msg;
}

fn extract_regex<'a>(req: &'a Vec<&str>, start: usize) -> Vec<&'a str> {
    let mut req_msg: Vec<&str> = Vec::new();
    for s in start..req.len() {
        req_msg.push(req[s]);
    }
    return req_msg;
}

impl<'a> Message<'a> {
    pub fn new(time: u64, key: &'a str) -> Self {
        return Message { key, time };
    }
}

impl<'a> Clone for Message<'a> {
    fn clone(&self) -> Self {
        return Message {
            key: self.key,
            time: self.time,
        };
    }
}

impl<'a> PartialEq for Message<'a> {
    fn eq(&self, other: &Self) -> bool {
        return self.key == other.key && self.time == other.time;
    }
}

impl<'a> RedisType<'a> {
    pub fn is_delay(&self) -> bool {
        match self {
            RedisType::Delay(_) => true,
            _ => false,
        }
    }
}

impl<'a> Display for RedisType<'a> {
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
                for i in 0..elements.len() {
                    res.push_str(&elements[i].to_string());
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
            RedisType::Delay(_) => {
                return write!(f, "$+OK\r\n");
            }
        }
    }
}

impl<'a> PartialEq for RedisType<'a> {
    fn eq(&self, other: &Self) -> bool {
        match &self {
            RedisType::SimpleString(msg) => match other {
                RedisType::SimpleString(msg2) => {
                    if msg == msg2 {
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
            RedisType::Error(msg) => match other {
                RedisType::Error(msg2) => {
                    if msg == msg2 {
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
            RedisType::Integer(msg) => match other {
                RedisType::Integer(msg2) => {
                    if msg == msg2 {
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
            RedisType::BulkString(msg) => match other {
                RedisType::BulkString(msg2) => {
                    if msg == msg2 {
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
            RedisType::Array(elements) => match other {
                RedisType::Array(elements2) => {
                    if elements == elements2 {
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
            RedisType::Null => match other {
                RedisType::Null => true,
                _ => false,
            },
            RedisType::Boolean(msg) => match other {
                RedisType::Boolean(msg2) => {
                    if msg == msg2 {
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
            RedisType::NullBulk => match other {
                RedisType::NullBulk => true,
                _ => false,
            },

            RedisType::Delay(_) => match other {
                RedisType::Delay(_) => true,
                _ => false,
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
        let data = Arc::new(Mutex::new(Database::new()));
        let msg: String = String::from("*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n");
        let ans = get_redis_response(&msg, data).await.unwrap();
        assert_eq!(ans, RedisType::BulkString("hey".to_string()));
    }

    #[tokio::test]
    async fn get_conf_command_test() {
        let data = Arc::new(Mutex::new(Database::new()));
        data.lock().await.add("dir", "/tmp/redis-files");

        data.lock().await.add("dbfilename", "dump.rdb");

        let msg: String = String::from("*3\r\n$6\r\nCONFIG\r\n$3\r\nGET\r\n$3\r\ndir\r\n");
        let ans = get_redis_response(&msg, Arc::clone(&data)).await.unwrap();
        assert_eq!(
            ans,
            RedisType::Array(Box::new(vec![
                RedisType::BulkString("dir".to_string()),
                RedisType::BulkString("/tmp/redis-files".to_string())
            ]))
        );

        let msg: String = String::from("*3\r\n$6\r\nCONFIG\r\n$3\r\nGET\r\n$3\r\ndbfilename\r\n");
        let ans = get_redis_response(&msg, data).await.unwrap();
        assert_eq!(
            ans,
            RedisType::Array(Box::new(vec![
                RedisType::BulkString("dbfilename".to_string()),
                RedisType::BulkString("dump.rdb".to_string())
            ]))
        );
    }

    #[tokio::test]
    async fn get_command_test() {
        let data = Arc::new(Mutex::new(Database::new()));
        data.lock().await.add("foo", "bar");

        let msg: String = String::from("*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n");
        let ans = get_redis_response(&msg, data).await.unwrap();
        assert_eq!(ans, RedisType::BulkString("bar".to_string()));
    }

    #[tokio::test]
    async fn set_command_test() {
        let data = Arc::new(Mutex::new(Database::new()));

        let msg: String = String::from("*3\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");
        let ans = get_redis_response(&msg, Arc::clone(&data)).await.unwrap();
        assert_eq!(ans, RedisType::SimpleString("OK"));

        assert_eq!(*(data.lock().await.get("foo").unwrap()), "bar".to_string());
    }

    #[test]
    fn command_convert_test() {
        let sen1: String = String::from("ECHO Hello world");
        let res1: Vec<&str> = split_command(&sen1).unwrap();

        assert_eq!(
            RedisType::BulkString(res1[1].to_string()),
            RedisType::BulkString("Hello world".to_string())
        );
    }

    #[test]
    fn simple_string_test() {
        let my_simple: RedisType = RedisType::SimpleString("OK");
        assert_eq!(my_simple.to_string(), "+OK\r\n");
    }

    #[test]
    fn error_test() {
        let my_error: RedisType = RedisType::Error("Err unknown command");
        assert_eq!(my_error.to_string(), "-Err unknown command\r\n");
    }

    #[test]
    fn integer_test() {
        let my_int: RedisType = RedisType::Integer(String::from("0"));
        assert_eq!(my_int.to_string(), ":0\r\n");
    }

    #[test]
    fn bulk_test() {
        let my_bulk: RedisType = RedisType::BulkString("Hello world".to_string());
        assert_eq!(my_bulk.to_string(), "$11\r\nHello world\r\n");
    }

    #[test]
    fn array_test_string() {
        let array: Box<Vec<RedisType>> = Box::new(vec![
            RedisType::BulkString("Hello".to_string()),
            RedisType::BulkString("world".to_string()),
        ]);

        let my_array_type: RedisType = RedisType::Array(array);
        assert_eq!(
            my_array_type.to_string(),
            "*2\r\n$5\r\nHello\r\n$5\r\nworld\r\n"
        );
    }

    #[test]
    fn array_test_int() {
        let array: Box<Vec<RedisType>> = Box::new(vec![
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
