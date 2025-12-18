use pethit_storage::SimpleStorage;
use alloy_primitives::{B256, keccak256};

/// A Transaction is a request to change the state.
/// In Iteration 1, a transaction is simply:
/// "Please save this Value under this Key."
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Transaction {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

impl Transaction {
    pub fn hash(self) -> B256 {
        // Concatenate the tx data. TODO: concatenate with RLP
        let mut data = self.key.clone();
        data.extend(&self.value);

        // Hash the data with keccak256.
        keccak256(data)
    }
}
#[derive(Debug)]
// The ExecutionEngine holds no state/data, it only holds the logic.
pub struct ExecutionEngine;

impl Default for ExecutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionEngine {
    // Since storage wil be muted, ownership of storage is moved into the engine for simplicity
    pub fn new() -> Self {
        ExecutionEngine {}
    }

    /// The Core Function: execute a transaction.
    /// Takes a mutable borrow of the storage (`&mut SimpleStorage`)and applies the tx to the storage.
    pub fn execute(storage: &mut SimpleStorage, tx: &Transaction) {
        // Signatures will be checked here. Now it trust tx and write to db
        storage.put(tx.key.clone(), tx.value.clone());
    }

    // A helper to see the current state
    pub fn get_state(storage: &SimpleStorage, key: &[u8]) -> Option<Vec<u8>> {
        storage.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_executes_a_transaction() {
        // setup
        let mut storage = SimpleStorage::new();

        // create tx
        let tx = Transaction {
            key: b"This is the key".to_vec(),
            value: b"This is the value".to_vec(),
        };

        // Run the tx
        ExecutionEngine::execute(&mut storage, &tx);

        // Verify the state changed
        let result = ExecutionEngine::get_state(&storage, b"This is the key");
        assert_eq!(result, Some(b"This is the value".to_vec()));
    }

    #[test]
    fn test_transaction_hashing() {
        // create tx
        let tx = Transaction {
            key: b"This is the key".to_vec(),
            value: b"This is the value".to_vec(),
        };

        let hash =tx.hash();
        println!("Tx Hash: {}", hash);

        // Assert it is 32 bytes
        assert_eq!(hash.len(), 32)

    }
}
