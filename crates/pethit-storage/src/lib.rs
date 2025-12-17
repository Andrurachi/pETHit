use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// A simple in-memory Key-Value database.
/// This struct holds one piece of data: the HashMap.
#[derive(Debug, Default)]
pub struct SimpleStorage {
    pub db: HashMap<Vec<u8>, Vec<u8>>,
}

impl SimpleStorage {
    /// Creates a new, empty storage instance.
    pub fn new() -> Self {
        // Default::default() tells Rust how to create a default SimpleStorage (one with an empty HashMap).
        Self::default()
    }

    /// Insert a new key-value pair in the database.
    /// We use `Vec<u8>` (a vector of bytes) because this is the
    /// raw data format blockchains use for everything.
    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.db.insert(key, value);
    }

    /// Retrieves a value from the database given the key.
    /// It returns Option because the key might not exist.
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.db.get(key).cloned()
    }
}

/// The Thread-Safe Public Interface.
#[derive(Clone)]
pub struct SharedStorage {
    inner: Arc<Mutex<SimpleStorage>>,
}

impl SharedStorage {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(SimpleStorage::new())),
        }
    }

    /// Insert a new key-value pair in the database.
    pub fn put(&self, key: Vec<u8>, value: Vec<u8>) {
        let mut db = self.inner.lock().unwrap();
        db.put(key, value);
    }

    // The RPC uses this to check balances.
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let db = self.inner.lock().unwrap();
        db.get(&key)
    }

    // The "Guard" method the Miner uses to modify the db.
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut SimpleStorage),
    {
        let mut db = self.inner.lock().unwrap();
        f(&mut db);
    }
}

// Tests
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_puts_and_gets() {
        // Create an instance of SimpleStorage
        let storage = SharedStorage::new();

        // Create a kv sample in vec<u8>
        let key1 = b"This is the key".to_vec();
        let value1 = b"some value for the key".to_vec();

        // Put a new kv int our storage
        storage.put(key1.clone(), value1.clone());

        // Get a value stored given the key
        let retrieved_value = storage.get(&key1);

        // Assert that the returned value is the same as the original value
        assert_eq!(retrieved_value, Some(value1));
    }

    #[test]
    fn it_returns_none_for_missing_key() {
        let storage = SimpleStorage::new();
        let key_missing = b"missing_key".to_vec();

        let retrieved_value = storage.get(&key_missing);

        assert_eq!(retrieved_value, None);
    }
}
