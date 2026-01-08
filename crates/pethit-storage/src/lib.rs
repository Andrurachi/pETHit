use alloy_primitives::{Address, U256};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// Represents a single user's state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Account {
    pub nonce: u64,
    pub balance: U256,
}

/// This struct holds the information of all accounts
#[derive(Debug, Default)]
pub struct SimpleStorage {
    pub accounts: HashMap<Address, Account>,
}

impl SimpleStorage {
    /// Creates a new, empty storage instance.
    pub fn new() -> Self {
        // Default::default() tells Rust how to create a default SimpleStorage.
        Self::default()
    }

    /// Helper to update an account
    pub fn set_account(&mut self, addr: Address, account: Account) {
        self.accounts.insert(addr, account);
    }

    /// Helper to get an account info
    pub fn get_account(&self, addr: &Address) -> Account {
        self.accounts.get(addr).cloned().unwrap_or_default()
    }
}

/// The Thread-Safe Public Interface.
#[derive(Clone)]
pub struct SharedStorage {
    inner: Arc<Mutex<SimpleStorage>>,
}

impl Default for SharedStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedStorage {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(SimpleStorage::new())),
        }
    }

    /// Update an account
    pub fn set_account(&self, addr: Address, account: Account) {
        let mut accounts = self.inner.lock().unwrap();
        accounts.set_account(addr, account);
    }

    // RPC uses this to check balances.
    // Returns an account given and address
    pub fn get_account(&self, addr: Address) -> Account {
        let accounts = self.inner.lock().unwrap();
        accounts.get_account(&addr)
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

// TODO: Update tests with the new storage.
// // Tests
// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn it_puts_and_gets() {
//         // Create an instance of SimpleStorage
//         let storage = SharedStorage::new();

//         // Create a kv sample in vec<u8>
//         let key1 = b"This is the key".to_vec();
//         let value1 = b"some value for the key".to_vec();

//         // Put a new kv int our storage
//         storage.put(key1.clone(), value1.clone());

//         // Get a value stored given the key
//         let retrieved_value = storage.get(&key1);

//         // Assert that the returned value is the same as the original value
//         assert_eq!(retrieved_value, Some(value1));
//     }

//     #[test]
//     fn it_returns_none_for_missing_key() {
//         let storage = SimpleStorage::new();
//         let key_missing = b"missing_key".to_vec();

//         let retrieved_value = storage.get(&key_missing);

//         assert_eq!(retrieved_value, None);
//     }
// }
