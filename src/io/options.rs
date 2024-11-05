use std::collections::{HashMap, HashSet};

pub struct Options {
    values: HashMap<String, String>,
    flags: HashSet<String>,
}

impl Options {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            flags: HashSet::new(),
        }
    }

    pub fn add_flag(&mut self, flag: String) {
        self.flags.insert(flag);
    }

    pub fn add_value(&mut self, key: String, value: String) {
        self.values.insert(key, value);
    }

    pub fn has_flag(&self, flag: String) -> bool {
        self.flags.contains(&flag)
    }

    pub fn get_value(&self, key: String) -> Option<&String> {
        self.values.get(&key)
    }
}

