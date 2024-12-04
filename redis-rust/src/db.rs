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

    pub fn add(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<String> {
        return Some(self.data.get(key).unwrap().clone());
    }

    pub fn remove(&mut self, key: &str) {
        self.data.remove(key);
    }

    pub fn try_get(&self, key: &str) -> Option<()> {
        match self.data.get(key) {
            Some(_) => Some(()),
            None => None,
        }
    }

    pub fn get_keys(&self) -> Vec<String> {
        return self.data.iter().skip(2).map(|(k, _)| k.clone()).collect();
    }
}
