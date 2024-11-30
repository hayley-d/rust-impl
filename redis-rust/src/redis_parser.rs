use std::fmt::Display;

pub enum RedisType {
    SimpleString(String),
    Error(String),
    Integer(String),
    BulkString(String),
    Array(Box<[RedisType]>),
    Null,
    Boolean(bool),
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

#[cfg(test)]
mod tests {
    use super::*;

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
