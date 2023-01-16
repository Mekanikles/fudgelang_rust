use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use std::fmt;

pub use u64 as StringKey;

#[derive(Copy, Clone)]
pub struct StringRef {
    pub key: StringKey,
}

impl<'a> fmt::Debug for StringRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return f.write_fmt(format_args!("<string key: {}>", self.key));
    }
}

pub struct StringStore {
    strings: HashMap<u64, String>,
}

impl StringStore {
    pub fn new() -> Self {
        StringStore {
            strings: HashMap::new(),
        }
    }

    fn get_key(string: &str) -> StringKey {
        let mut hasher = DefaultHasher::new();
        string.hash(&mut hasher);
        return hasher.finish();
    }

    pub fn insert(&mut self, string: &str) -> StringRef {
        let key = Self::get_key(string);
        if !self.strings.contains_key(&key) {
            self.strings.insert(key, string.into());
        }

        return StringRef { key };
    }

    pub fn get(&self, stringref: &StringRef) -> Option<&String> {
        self.strings.get(&stringref.key)
    }
}
