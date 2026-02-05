pETHit: A Simplified Ethereum Implementation
===

This contains my design notes, research, and implementation details for `pETHit`.

This project is going to be developed in iterations in which we start with the simplest version of Ethereum, iterate to a better one with more complex components. 

This project is intended for learning the Ethereum Protocol in depth, selecting my favorite subsystem and improving my Rust skills.

## Project Components

- [Storage](https://hackmd.io/@Wre4AEjBTny7MFhlY4ci4g/rJF245tlbe)
- [Execution](https://hackmd.io/@Wre4AEjBTny7MFhlY4ci4g/BkgLSTwqlbg)
- [RPC API](https://hackmd.io/@Wre4AEjBTny7MFhlY4ci4g/HkvK6R6eZl)
- [TxPool](https://hackmd.io/@Wre4AEjBTny7MFhlY4ci4g/H1VGfMS-be)
- [consensus](/gVExmqs6S3iyeiV3UYTqGw)
- [Node](https://hackmd.io/@Andrurachi/SyCzyUMXZe)
- [Wallet](https://hackmd.io/@Andrurachi/ByYvm-bP-g)

---

## The First Iteration Plan (v0.1.0): The Walking Skeleton
We make the first version of each one extremely simple. As we should haha.

- **Storage**: A simple key-value database. (We can ignore the complex Trie for now).
- **Execution**: A simple "state transition function" that defines a Transaction and a State. The function just takes a state and a tx and produces a new_state.
- **RPC API**: A tiny web server that can accept a "send transaction" command.
- **Transaction Pool**: A simple in-memory list that holds transactions received from the RPC.
- **Consensus**: A simple "miner" that runs in a loop. Every 5 seconds, it grabs transactions from the TxPool, runs them through the Execution function, creates a Block, and saves it to Storage. (Proof-of-Authority).
- **Node:** Integration of previous subsystems.

---

## The Second Iteration Plan (v0.2.0): The Actual Blockchain


### The Block Linking 

Currently, all the blocks are separated items. In this iteration, hashing is going to be used to create a way of linking every block. This way, if a block is changed it is going to break the hash of all blocks that comes after it. 

**Data Structure Changes:**

* **Block:** A block is no longer just "transactions". It needs metadata to verify its place in history.
* **`id` (u64):** The height (1, 2, 3...).
* **`transactions` (`Vec<Transaction>`):** A vector with all transactions in this block.
* **`parent_hash` (B256):** The fingerprint of the previous block.
* **`k_hash` (B256):** The fingerprint of this block (computed from the fields above). A `SealedBlock` is composed by a block and its k_hash



### Logic Changes

**A. Miner's New Job:**

* **Step 1:** Fetch transactions.
* **Step 2:** Look at the **Last Block** in history to get its `hash`.
* **Step 3:** Create the new block, setting `parent_hash` = `last_block.k_hash`.
* **Step 4:** "Seal" the block by calculating its own unique `hash`.

**B. The Genesis Block:**

* Block #1 needs a parent. The "Genesis Block" (Block #0) will have a `parent_hash` of `0x0000...0000`.


---

## The Third Iteration Plan (v0.3.0): The Identity Update

The goal of this iteration is to introduce **Cryptography** and **Identity**. This way the system moves from "Anyone can write anything" to "You can only spend what you own."

### Key Features
1.  **Cryptography (ECDSA):** `k256` was integrated  to handle Elliptic Curve Digital Signature Algorithm (secp256k1).
2.  **Wallet CLI:** A separate binary (`pethit-wallet`) that acts as a client. It generates private keys, signs transactions locally, and broadcasts them to the node.
3.  **Serialization (RLP):** Recursive Length Prefix (RLP) encoding was implemented. This allows complex structs to be turned into bytes for network transport, hashing and storing.
4.  **Stateful Accounts:** The storage no longer holds arbitrary strings. It now holds `Account` structs (`nonce` and `balance`) associated with an address.
5.  **Replay Protection:** `nonce` was introduced to prevent the same transaction from being executed twice.

### The New Flow
1.  **Wallet:** Generates a key pair -> Queries Node for Nonce -> Signs Tx (RLP + Hash) -> Sends to Node.
2.  **RPC:** Receives Hex -> Decodes RLP -> Verifies Signature -> Adds to Pool.
3.  **Miner:** Picks Tx -> Executes -> Updates Global State (Balances/Nonces).

---
