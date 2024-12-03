use std::collections::HashMap;

pub struct Database {
    data: HashMap<String, String>,
}

impl Database {
    pub fn new() -> Self {
        return Database {
            data: HashMap::new(),
        };
    }

    pub fn add(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: String) -> Option<&String> {
        return self.data.get(&key);
    }

    pub fn remove(&mut self, key: String) {
        self.data.remove(&key);
    }

    pub fn try_get(&self, key: &str) -> Option<()> {
        match self.data.get(key) {
            Some(_) => Some(()),
            None => None,
        }
    }

    pub fn get_keys(&self) -> Vec<String> {
        return self
            .data
            .iter()
            .skip(2)
            .map(|(k, _)| k.to_string())
            .collect();
    }
}
