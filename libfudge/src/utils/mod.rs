pub mod stringstore;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct StringKey {
    pub key: u64,
    #[cfg(debug_assertions)]
    debugname: String,
}

impl StringKey {
    fn create_key(string: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        string.hash(&mut hasher);
        return hasher.finish();
    }

    #[cfg(debug_assertions)]
    pub fn from_str(string: &str) -> StringKey {
        StringKey {
            key: Self::create_key(string),
            debugname: string.to_string(),
        }
    }

    #[cfg(not(debug_assertions))]
    pub fn from_str(string: &str) -> StringKey {
        StringKey {
            key: Self::create_key(string),
        }
    }

    #[cfg(debug_assertions)]
    pub fn from_hash(hash: &u64) -> StringKey {
        StringKey {
            key: *hash,
            debugname: "<Raw Hash>".into(),
        }
    }

    #[cfg(not(debug_assertions))]
    pub fn from_hash(hash: &u64) -> StringKey {
        StringKey { key: hash }
    }
}

impl Hash for StringKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl PartialEq for StringKey {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl Eq for StringKey {}

impl std::fmt::Display for StringKey {
    #[cfg(debug_assertions)]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.debugname)
    }

    #[cfg(not(debug_assertions))]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.key)
    }
}
