use alloy_primitives::B256;
use pethit_execution::Transaction;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// A simple error type
pub type PoolError = String;

/// This doesn't know about threads, just data.
struct TxPool {
    // The Transaction itself is the Key (for deduplication).
    // The Value is empty unit type (no needed extra metadata yet).
    transactions: HashMap<B256, Transaction>,
}

impl TxPool {
    fn new() -> Self {
        Self {
            transactions: HashMap::new(),
        }
    }

    fn add(&mut self, k_hash: &B256, tx: &Transaction) {
        // HashMap::insert automatically overwrites if key exists (deduplication)
        self.transactions.insert(*k_hash, tx.clone());
    }

    fn get_all(&self) -> Vec<Transaction> {
        // Return a cloned list of all transactions
        self.transactions.values().cloned().collect()
    }

    fn clear(&mut self) {
        // Clears the pool (called after a block is mined)
        self.transactions.clear();
    }
}

/// The Thread-Safe Public Interface.
/// This is what we pass around the application.
#[derive(Clone)]
pub struct SharedTxPool {
    // Arc allows multiple owners.
    // Mutex allows exclusive access (mutability).
    inner: Arc<Mutex<TxPool>>,
}

impl Default for SharedTxPool {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedTxPool {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(TxPool::new())),
        }
    }

    /// Adds a transaction to the pool in a thread-safe way.
    pub fn add(&self, k_hash: &B256, tx: &Transaction) -> Result<(), PoolError> {
        // Lock the Mutex
        let mut pool = self.inner.lock().map_err(|_| "Lock poisoned".to_string())?;
        // Call the internal function
        pool.add(k_hash, tx);

        Ok(())
    }

    /// Retrieves all transactions.
    pub fn get_all_transactions(&self) -> Vec<Transaction> {
        let pool = self.inner.lock().unwrap();
        pool.get_all()
    }

    /// Clears the pool
    pub fn clear(&self) {
        let mut pool = self.inner.lock().unwrap();
        pool.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_add_transaction() {
        let pool = SharedTxPool::new();
        let tx = Transaction {
            key: b"key".to_vec(),
            value: b"value".to_vec(),
        };
        let k_hash = tx.hash();

        // Add it
        pool.add(&k_hash, &tx).unwrap();

        // Check it exists
        let all_txs = pool.get_all_transactions();
        assert_eq!(all_txs.len(), 1);
        assert_eq!(all_txs[0], tx);
    }

    #[test]
    fn test_deduplication() {
        let pool = SharedTxPool::new();
        let tx = Transaction {
            key: b"same".to_vec(),
            value: b"same".to_vec(),
        };
        let k_hash = tx.hash();

        // Add the exact same tx twice
        pool.add(&k_hash, &tx).unwrap();
        pool.add(&k_hash, &tx).unwrap();

        // Should only have 1 in storage
        let all_txs = pool.get_all_transactions();
        assert_eq!(all_txs.len(), 1);
    }

    #[test]
    fn test_concurrency_multiple_threads() {
        let pool = SharedTxPool::new();
        let mut handles = vec![];

        // Spawn 10 threads
        for i in 0..10 {
            // Clone the "handle" to the pool for this thread
            let pool_clone = pool.clone();

            let handle = thread::spawn(move || {
                // Create a unique tx (based on index)
                let tx = Transaction {
                    key: format!("key_{}", i).into_bytes(),
                    value: b"val".to_vec(),
                };
                let k_hash = tx.hash();

                pool_clone.add(&k_hash, &tx).unwrap();
            });

            handles.push(handle);
        }

        // Wait for all threads to finish
        for handle in handles {
            handle.join().unwrap();
        }

        // If locking works, we should have exactly 10 transactions.
        let all_txs = pool.get_all_transactions();
        assert_eq!(all_txs.len(), 10);
    }
}
