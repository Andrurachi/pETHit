use pethit_execution::Transaction;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Generic error type for simplicity
pub type PoolError = String;

/// The internal storage
/// We use HashMap where Key = the value itself and value = ()
struct Txpool {
    transactions: HashMap<Transaction, ()>,
}

impl Txpool {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
        }
    }

    /// Adds a transaction to the pool
    pub fn add(&mut self, tx: Transaction) -> Result<(), PoolError> {
        // If it exists, it updates it. If not, it inserts.
        // Already handles deduplication thanks to value = ()
        self.transactions.insert(tx, ());
        Ok(())
    }

    /// Return all transactions to be included in a block.
    pub fn get_all(&self) -> Vec<Transaction> {
        self.transactions.keys().cloned().collect()
    }

    /// Clears the pool (called after a block is mined)
    pub fn clear(&mut self) {
        self.transactions.clear();
    }
}

// "Public API" that is passed around
// Wraps the raw Txpool in an Arc<Mutex<>>
#[derive(Clone)]
pub struct ShareTxPool {
    inner: Arc<Mutex<Txpool>>,
}

impl ShareTxPool {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Txpool::new())),
        }
    }
    pub fn add(self, tx: Transaction) -> Result<(), PoolError> {
        // Lock the Mutex
        let mut pool = self.inner.lock().map_err(|_| "Lock poisoned".to_string())?;
        // Call the internal function
        pool.add(tx)
    }

    pub fn get_all_transactions(self) -> Vec<Transaction> {
        let pool = self.inner.lock().unwrap();
        pool.get_all()
    }

    pub fn clear(&self) {
        let mut pool = self.inner.lock().unwrap();
        pool.clear();
    }
}
