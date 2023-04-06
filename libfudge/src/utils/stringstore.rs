use crate::utils::*;

pub type StringStore = objectstore::HashedObjectStore<StringKey, String>;

impl objectstore::HashedStoreKey<String> for StringKey {
    fn from_obj(object: &String) -> Self {
        StringKey::from_str(object.as_str())
    }
}
