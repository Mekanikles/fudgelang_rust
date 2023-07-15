use std::collections::HashMap;
use std::hash::Hash;

pub trait ObjectStore<KeyT, ObjectT> {
    type Key;

    fn add(&mut self, object: ObjectT) -> KeyT;
    fn has(&self, key: &KeyT) -> bool;
    fn get(&self, key: &KeyT) -> &ObjectT;
    fn get_mut(&mut self, key: &KeyT) -> &mut ObjectT;
    fn try_get(&self, key: &KeyT) -> Option<&ObjectT>;
    fn try_get_mut(&mut self, key: &KeyT) -> Option<&mut ObjectT>;
}

#[derive(Debug)]
pub struct IndexedObjectStore<ObjectT> {
    objects: Vec<ObjectT>,
}

impl<ObjectT> IndexedObjectStore<ObjectT> {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn keys(&self) -> std::ops::Range<usize> {
        0..self.objects.len()
    }

    pub fn values(&self) -> &Vec<ObjectT> {
        &self.objects
    }

    pub fn values_mut(&mut self) -> &mut Vec<ObjectT> {
        &mut self.objects
    }

    pub fn remove_vec(&mut self) -> Vec<ObjectT> {
        std::mem::replace(&mut self.objects, Vec::new())
    }

    pub fn into_iter(&mut self) -> <Vec<ObjectT> as IntoIterator>::IntoIter {
        let objects = std::mem::replace(&mut self.objects, Vec::new());
        objects.into_iter()
    }
}

impl<ObjectT> ObjectStore<usize, ObjectT> for IndexedObjectStore<ObjectT> {
    type Key = usize;

    fn add(&mut self, object: ObjectT) -> Self::Key {
        self.objects.push(object);
        self.objects.len() - 1
    }
    fn has(&self, key: &Self::Key) -> bool {
        self.objects.len() > *key
    }
    fn get(&self, key: &Self::Key) -> &ObjectT {
        &self.objects[*key]
    }
    fn get_mut(&mut self, key: &Self::Key) -> &mut ObjectT {
        self.objects.get_mut(*key).unwrap()
    }
    fn try_get(&self, key: &Self::Key) -> Option<&ObjectT> {
        self.objects.get(*key)
    }
    fn try_get_mut(&mut self, key: &Self::Key) -> Option<&mut ObjectT> {
        self.objects.get_mut(*key)
    }
}

pub trait HashedStoreKey<ObjectT>: Hash + Eq + Clone {
    fn from_obj(object: &ObjectT) -> Self;
}

#[derive(Debug)]
pub struct HashedObjectStore<KeyT: HashedStoreKey<ObjectT>, ObjectT: Eq> {
    objects: HashMap<KeyT, ObjectT>,
}

impl<KeyT: HashedStoreKey<ObjectT>, ObjectT: Eq> HashedObjectStore<KeyT, ObjectT> {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }

    pub fn keys(&self) -> std::collections::hash_map::Keys<'_, KeyT, ObjectT> {
        self.objects.keys()
    }

    pub fn values(&self) -> std::collections::hash_map::Values<'_, KeyT, ObjectT> {
        self.objects.values()
    }
}

impl<KeyT: HashedStoreKey<ObjectT>, ObjectT: Eq> ObjectStore<KeyT, ObjectT>
    for HashedObjectStore<KeyT, ObjectT>
{
    type Key = KeyT;

    fn add(&mut self, object: ObjectT) -> KeyT {
        let key = KeyT::from_obj(&object);

        if self.objects.contains_key(&key) {
            assert!(object == *self.get(&key));
            return key;
        }

        self.objects.insert(key.clone(), object);

        return key;
    }
    fn has(&self, key: &KeyT) -> bool {
        self.objects.contains_key(key)
    }
    fn get(&self, key: &KeyT) -> &ObjectT {
        &self.objects.get(key).unwrap()
    }
    fn get_mut(&mut self, key: &KeyT) -> &mut ObjectT {
        self.objects.get_mut(key).unwrap()
    }
    fn try_get(&self, key: &KeyT) -> Option<&ObjectT> {
        self.objects.get(key)
    }
    fn try_get_mut(&mut self, key: &KeyT) -> Option<&mut ObjectT> {
        self.objects.get_mut(key)
    }
}
