use std::collections::HashMap;

#[derive(Default)]
pub struct Store {
    pub store: HashMap<String, String>,
}

impl Store {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn read(&self, key: &str) -> Option<&String> {
        self.store.get(key)
    }

    pub fn write(&mut self, key: String, value: String) {
        self.store.insert(key, value);
    }
}
