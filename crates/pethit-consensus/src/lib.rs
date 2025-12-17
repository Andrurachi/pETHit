use pethit_execution::{ExecutionEngine, Transaction};
use pethit_storage::SharedStorage;
use pethit_txpool::SharedTxPool;
use std::{thread, time::Duration};

#[derive(Debug, Clone)]
pub struct Block {
    pub id: u64,
    pub transactions: Vec<Transaction>,
}

pub struct Miner {
    txpool: SharedTxPool,
    storage: SharedStorage,
    blockchain: Vec<Block>,
    block_num: u64,
}

impl Miner {
    /// The Miner is initialized with existing handles to the Pool and Storage.
    pub fn new(txpool: SharedTxPool, storage: SharedStorage) -> Self {
        Self {
            txpool,
            storage,
            blockchain: Vec::new(),
            block_num: 0,
        }
    }

    /// The "Heartbeat" loop.
    /// 'mut self' because we update 'block_num' and 'blockchain'.
    pub fn start_mining(mut self) {
        println!("Miner initialized and starting heartbeat...");
        loop {
            thread::sleep(Duration::from_secs(5));
            self.mine_block();
        }
    }

    fn mine_block(&mut self) {
        // Pull transactions from the shared pool
        let txs = self.txpool.get_all_transactions();
        // If there are txs, update the STATE
        if !txs.is_empty() {
            // .update() pattern is used to lock the DB once and run all transactions through the Engine.
            let txs_to_execute = txs.clone();
            self.storage.update(|raw_db| {
                for tx in txs_to_execute {
                    ExecutionEngine::execute(raw_db, &tx);
                }
            });
        }

        // Create the Block
        self.block_num += 1;
        let block = Block {
            id: self.block_num,
            transactions: txs,
        };

        println!(
            "Mined Block #{} with {} txs",
            block.id,
            block.transactions.len()
        );

        // Save to history and clear the pool
        self.blockchain.push(block);
        self.txpool.clear();
    }
}
