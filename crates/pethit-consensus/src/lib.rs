use alloy_primitives::{B256, keccak256};
use pethit_execution::{ExecutionEngine, Transaction};
use pethit_storage::SharedStorage;
use pethit_txpool::SharedTxPool;
use std::{thread, time::Duration};

#[derive(Debug, Clone)]
pub struct Block {
    pub id: u64,
    pub transactions: Vec<Transaction>,
    pub parent_hash: B256,
}

impl Block {
    pub fn hash(&self) -> B256 {
        let mut data = Vec::new();
        data.extend_from_slice(&self.id.to_be_bytes());
        data.extend_from_slice(self.parent_hash.as_slice());

        for tx in &self.transactions {
            data.extend_from_slice(tx.hash().as_slice());
        }

        keccak256(data)
    }

    pub fn seal(self) -> SealedBlock {
        let hashed_block = self.hash();

        SealedBlock {
            block: self,
            k_hash: hashed_block,
        }
    }
}

// Includes the block hash (removes the need to use placeholder hash and mut block)
#[derive(Debug, Clone)]
pub struct SealedBlock {
    pub block: Block,
    pub k_hash: B256,
}

impl std::ops::Deref for SealedBlock {
    type Target = Block;
    fn deref(&self) -> &Self::Target {
        &self.block
    }
}

pub struct Miner {
    txpool: SharedTxPool,
    storage: SharedStorage,
    blockchain: Vec<SealedBlock>,
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

        // Create genesis block
        let genesis = Block {
            id: self.block_num,
            transactions: Vec::new(),
            parent_hash: B256::ZERO,
        }
        .seal();
        self.blockchain.push(genesis);

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
        let parent_block = self.blockchain.last().unwrap();
        let sealed_block = Block {
            id: self.block_num,
            transactions: txs,
            parent_hash: parent_block.k_hash,
        }
        .seal();

        println!(
            "Mined Block #{} (Hash: {}) with {} txs",
            sealed_block.id,
            sealed_block.k_hash,
            sealed_block.transactions.len()
        );

        // Save to history and clear the pool
        self.blockchain.push(sealed_block);
        self.txpool.clear();
    }
}
