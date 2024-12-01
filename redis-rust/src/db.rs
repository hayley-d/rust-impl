use std::collections::HashMap;

pub struct Data {
    data: HashMap<String, String>,
}

impl Data {
    pub fn new() -> Self {
        return Data {
            data: HashMap::new(),
        };
    }

    pub fn add(&mut self, key: String, value: String) {
        println!("Adding key: {} and value: {}", key, value);
        self.data.insert(key, value);
    }

    pub fn get(&self, key: String) -> Option<&String> {
        println!("Getting {} from the hashmap", key);
        return self.data.get(&key);
    }

    pub fn remove(&mut self, key: String) {
        self.data.remove(&key);
    }
}
