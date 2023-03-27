use std::collections::HashMap;

use crate::utils::StringKey;

pub struct StringStore {
    strings: HashMap<StringKey, String>,
}

impl StringStore {
    pub fn new() -> Self {
        StringStore {
            strings: HashMap::new(),
        }
    }

    pub fn insert(&mut self, string: &str) -> StringKey {
        let key = StringKey::from_str(string);

        if !self.strings.contains_key(&key) {
            self.strings.insert(key.clone(), string.into());
        }

        return key;
    }

    pub fn get(&self, key: &StringKey) -> Option<&String> {
        self.strings.get(&key)
    }
}
