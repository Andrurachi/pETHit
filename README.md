pETHit: A Simplified Ethereum Implementation
===

This project is going to be developed in iterations in which we start with the simplest version of Ethereum, iterate to a better one with more complex components, then select my favorite subsystem and make a robust version of the selected one. 

This project other than just learning the Ethereum Protocol in depth and improving my Rust skills will work as a layout for future proposal experiments to the Protocol.

## Project Components

- [Storage](https://hackmd.io/@Wre4AEjBTny7MFhlY4ci4g/rJF245tlbe)
- [Execution](https://hackmd.io/@Wre4AEjBTny7MFhlY4ci4g/BkgLSTwqlbg)
- [RPC API](https://hackmd.io/@Wre4AEjBTny7MFhlY4ci4g/HkvK6R6eZl)
- [TxPool](https://hackmd.io/@Wre4AEjBTny7MFhlY4ci4g/H1VGfMS-be)
- [consensus](/gVExmqs6S3iyeiV3UYTqGw)
- [Node](https://hackmd.io/@Andrurachi/SyCzyUMXZe)

---

## The First Iteration Plan (v0.1.0): The Walking Skeleton
We make the first version of each one extremely simple. As we should haha.

- **Storage**: A simple key-value database. (We can ignore the complex Trie for now).

- **Execution**: A simple "state transition function" that defines a Transaction and a State. The function just takes a state and a tx and produces a new_state (e.g., just debiting and crediting balances).

- **RPC API**: A tiny web server that can accept a "send transaction" command.

- **Transaction Pool**: A simple in-memory list that holds transactions received from the RPC.

- **Consensus**: A simple "miner" that runs in a loop. Every 5 seconds, it grabs transactions from the TxPool, runs them through the Execution function, creates a Block, and saves it to Storage. (Proof-of-Authority).


- **Node:** Integration of previous subsystems.

---

## The Second Iteration Plan (v0.2.0): The Actual Blockchain


### The Block Linking 

Currently, all the blocks are separated items. In this iteration, hashing is going to be used to create a way of linking every block. This way, if a block is changed it is going to break the hash of all blocks that comes after it. 

**Data Structure Changes:**

* **Block Header:** A block is no longer just "transactions". It needs metadata to verify its place in history.
* **`number` (u64):** The height (1, 2, 3...).
* **`parent_hash` (B256):** The fingerprint of the previous block.
* **`transactions_hash` (B256):** A fingerprint of all transactions in this block combined. (IN the future this will become the Merkle Root).
* **`hash` (B256):** The fingerprint of this block (computed from the fields above).



### Logic Changes

**A. Miner's New Job:**

* **Step 1:** Fetch transactions.
* **Step 2:** Look at the **Last Block** in history to get its `hash`.
* **Step 3:** Create the new block, setting `parent_hash` = `last_block.hash`.
* **Step 4:** "Seal" the block by calculating its own unique `hash`.

**B. The Genesis Block:**

* Block #1 needs a parent. The "Genesis Block" (Block #0) will have a `parent_hash` of `0x0000...0000`.


---

## The Third Iteration Plan (v0.3.0): The Identity Update


### 1. The Account

A random string like "project 1" won't be the key of transactions anymore. Now Ethereum Addresses will be used as keys in SimpleStorage.

### 2. The New Transaction Struct

The transaction struct needs to prove three things:

- **Who:** The sender (Address).
- **What:** The key and value.
- **Authorization:** The signature (a 65-byte array).

### 3. The Library: k256

To do this the industry way, k256 crate (used by Alloy and Reth) will be used. This crate handles the secp256k1 elliptic curve, which is the heart of Ethereum (and even Bitcoin).

### 4. The Logic Change (The Validation Step)

Currently the ExecutionEngine is a blind worker. It just does what it's told. In v0.3.0, the Engine will have a new rule:

"Before I save this value to the database, I will recover the public key ?from the signature. If the recovered address does not match the sender field, I will reject the transaction."

---
