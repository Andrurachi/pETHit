// pethit-execution/src/lib.rs

use pethit_storage::SimpleStorage;

/// A Transaction is a request to change the state.
/// In Iteration 1, a transaction is simply:
/// "Please save this Value under this Key."
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Transaction {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Debug)]
// The ExecutionEngine holds no state/data, it only holds the logic.
pub struct ExecutionEngine;

impl ExecutionEngine {
    // Since storage wil be muted, ownership of storage is moved into the engine for simplicity
    pub fn new() -> Self {
        ExecutionEngine {}
    }

    /// The Core Function: execute a transaction.
    /// Takes a mutable borrow of the storage (`&mut SimpleStorage`)and applies the tx to the storage.
    pub fn execute(storage: &mut SimpleStorage, tx: Transaction) {
        // Signatures will be checked here. Now it trust tx and write to db
        storage.put(tx.key, tx.value);
    }

    // A helper to see the current state
    // Lifetimes are needed because the return data is borrowed from SimpleStorage
    pub fn get_state<'a>(storage: &'a SimpleStorage, key: &[u8]) -> Option<&'a Vec<u8>> {
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
        ExecutionEngine::execute(&mut storage, tx);

        // Verify the state changed
        let result = ExecutionEngine::get_state(&storage, b"This is the key");
        assert_eq!(result, Some(&b"This is the value".to_vec()));
    }
}
