// pethit-execution/src/lib.rs

use pethit_storage::SimpleStorage;

/// A Transaction is a request to change the state.
/// In Iteration 1, a transaction is simply:
/// "Please save this Value under this Key."
pub struct Transaction {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Debug)]
pub struct ExecutionEngine {
    // The engine needs access to storage to do its job.
    storage: SimpleStorage,
}

impl ExecutionEngine {
    // Since storage wil be muted, ownership of storage is moved into the engine for simplicity
    pub fn new(storage: SimpleStorage) -> Self {
        Self { storage }
    }

    /// The Core Function: execute a transaction.
    /// It takes a transaction and applies it to the storage.
    pub fn execute(&mut self, tx: Transaction) {
        // Signatures will be checked here. Now it trust tx and write to db
        self.storage.put(tx.key, tx.value);
    }

    // A helper to see the current state
    pub fn get_state(&self, key: &[u8]) -> Option<&Vec<u8>> {
        self.storage.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_executes_a_transaction() {
        // setup
        let storage = SimpleStorage::new();
        let mut engine = ExecutionEngine::new(storage);

        // create tx
        let tx = Transaction {
            key: b"This is the key".to_vec(),
            value: b"This is the value".to_vec(),
        };

        // Run the tx
        engine.execute(tx);

        // Verify the state changed
        let result = engine.get_state(b"This is the key");
        assert_eq!(result, Some(&b"This is the value".to_vec()));
    }
}
