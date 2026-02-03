use alloy_primitives::{Address, U256};
use alloy_rlp::{Decodable, Encodable, RlpDecodable, RlpEncodable};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// Represents a single user's state.
#[derive(Debug, Clone, Default, PartialEq, Eq, RlpEncodable, RlpDecodable)]
pub struct Account {
    pub nonce: u64,
    pub balance: U256,
}

/// This struct holds the information of all accounts
#[derive(Debug, Default)]
pub struct SimpleStorage {
    pub accounts: HashMap<Vec<u8>, Vec<u8>>,
}

impl SimpleStorage {
    /// Creates a new, empty storage instance.
    pub fn new() -> Self {
        // Default::default() tells Rust how to create a default SimpleStorage.
        Self::default()
    }

    /// Helper to update an account
    pub fn set_account(&mut self, addr: Address, account: Account) {
        let key_bytes = addr.to_vec();
        // Account to RLP
        let mut value_bytes = Vec::new();
        account.encode(&mut value_bytes);
        self.accounts.insert(key_bytes, value_bytes);
    }

    /// Helper to get an account info
    pub fn get_account(&self, addr: &Address) -> Account {
        let key_bytes = addr.as_slice();

        match self.accounts.get(key_bytes) {
            Some(bytes) => {
                // RLP to Account
                Account::decode(&mut bytes.as_slice()).unwrap_or_default()
            }
            None => Account::default(),
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Address, U256};

    #[test]
    fn it_puts_and_gets_account() {
        let storage = SharedStorage::new();

        let addr = Address::ZERO;
        let account = Account {
            nonce: 5,
            balance: U256::from(100),
        };

        // Put
        storage.set_account(addr, account.clone());

        // Get
        let retrieved = storage.get_account(addr);

        // Check
        assert_eq!(retrieved.nonce, 5);
        assert_eq!(retrieved.balance, U256::from(100));
    }

    #[test]
    fn it_returns_default_for_missing_key() {
        let storage = SharedStorage::new();
        let addr = Address::ZERO;

        let retrieved = storage.get_account(addr);

        // Should be nonce 0, balance 0
        assert_eq!(retrieved.nonce, 0);
        assert_eq!(retrieved.balance, U256::from(0));
    }
}
