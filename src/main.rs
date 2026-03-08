//! # Mini Blockchain Implementation in Rust
//!
//! A from-scratch blockchain demonstrating:
//! - SHA-256 hashing
//! - Proof of Work (mining)
//! - Chain validation
//! - Transactions

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ─────────────────────────────────────────────
//  Transaction
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
}

impl Transaction {
    pub fn new(from: &str, to: &str, amount: f64) -> Self {
        Self {
            from: from.to_string(),
            to: to.to_string(),
            amount,
        }
    }
}

// ─────────────────────────────────────────────
//  Block
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub difficulty: usize,
}

impl Block {
    /// Create a new block (not yet mined)
    pub fn new(
        index: u64,
        transactions: Vec<Transaction>,
        previous_hash: String,
        difficulty: usize,
    ) -> Self {
        let timestamp = Utc::now().timestamp();
        let hash = String::new();

        let mut block = Self {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash,
            nonce: 0,
            difficulty,
        };

        // Mine immediately on creation
        block.mine();
        block
    }

    /// Calculate SHA-256 hash of this block's content
    pub fn calculate_hash(&self) -> String {
        let content = format!(
            "{}{}{}{}{}",
            self.index,
            self.timestamp,
            serde_json::to_string(&self.transactions).unwrap_or_default(),
            self.previous_hash,
            self.nonce
        );

        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Proof of Work — find nonce that makes hash start with `difficulty` zeros
    pub fn mine(&mut self) {
        let target = "0".repeat(self.difficulty);

        loop {
            let hash = self.calculate_hash();
            if hash.starts_with(&target) {
                self.hash = hash;
                return;
            }
            self.nonce += 1;
        }
    }

    /// Check if this block's hash is still valid
    pub fn is_valid(&self) -> bool {
        let recalculated = self.calculate_hash();
        recalculated == self.hash && self.hash.starts_with(&"0".repeat(self.difficulty))
    }
}

// ─────────────────────────────────────────────
//  Blockchain
// ─────────────────────────────────────────────

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: usize,
    pub pending_transactions: Vec<Transaction>,
    pub mining_reward: f64,
}

impl Blockchain {
    /// Create a new blockchain with genesis block
    pub fn new(difficulty: usize) -> Self {
        let genesis = Block::new(
            0,
            vec![Transaction::new("system", "genesis", 0.0)],
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            difficulty,
        );

        println!("⛏️  Genesis block mined! Hash: {}", &genesis.hash[..16]);

        Self {
            chain: vec![genesis],
            difficulty,
            pending_transactions: Vec::new(),
            mining_reward: 50.0,
        }
    }

    /// Get the latest block
    pub fn latest_block(&self) -> &Block {
        self.chain.last().unwrap()
    }

    /// Add a transaction to the pending pool
    pub fn add_transaction(&mut self, tx: Transaction) {
        println!(
            "📝 Transaction queued: {} → {} ({:.2} coins)",
            tx.from, tx.to, tx.amount
        );
        self.pending_transactions.push(tx);
    }

    /// Mine all pending transactions into a new block
    pub fn mine_pending_transactions(&mut self, miner_address: &str) {
        // Add mining reward transaction
        self.pending_transactions.push(Transaction::new(
            "system",
            miner_address,
            self.mining_reward,
        ));

        let previous_hash = self.latest_block().hash.clone();
        let index = self.chain.len() as u64;
        let txs = std::mem::take(&mut self.pending_transactions);

        print!("⛏️  Mining block #{index}...");
        std::io::Write::flush(&mut std::io::stdout()).ok();

        let block = Block::new(index, txs, previous_hash, self.difficulty);
        println!(" ✅ Hash: {}", &block.hash[..16]);

        self.chain.push(block);
    }

    /// Get balance of an address
    pub fn get_balance(&self, address: &str) -> f64 {
        let mut balance = 0.0f64;

        for block in &self.chain {
            for tx in &block.transactions {
                if tx.to == address {
                    balance += tx.amount;
                }
                if tx.from == address {
                    balance -= tx.amount;
                }
            }
        }

        balance
    }

    /// Validate the entire chain
    pub fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            // Check current block's hash
            if !current.is_valid() {
                println!("❌ Block #{} has invalid hash", i);
                return false;
            }

            // Check chain link
            if current.previous_hash != previous.hash {
                println!("❌ Block #{} has broken chain link", i);
                return false;
            }
        }
        true
    }

    /// Print chain summary
    pub fn print_chain(&self) {
        println!("\n{}", "═".repeat(60));
        println!("🔗 BLOCKCHAIN — {} blocks", self.chain.len());
        println!("{}", "─".repeat(60));

        for block in &self.chain {
            println!(
                "Block #{:<3} | Nonce: {:<8} | Txs: {:<3} | Hash: {}...",
                block.index,
                block.nonce,
                block.transactions.len(),
                &block.hash[..20]
            );
        }
        println!("{}", "═".repeat(60));
    }
}

// ─────────────────────────────────────────────
//  Main
// ─────────────────────────────────────────────

fn main() {
    println!("🦀 Mini Blockchain — Built in Rust\n");

    // Create blockchain (difficulty 3 = hash must start with "000")
    let mut chain = Blockchain::new(3);

    // Add transactions
    chain.add_transaction(Transaction::new("Alice", "Bob", 50.0));
    chain.add_transaction(Transaction::new("Bob", "Charlie", 25.0));

    // Mine block
    chain.mine_pending_transactions("Roseanne");

    // More transactions
    chain.add_transaction(Transaction::new("Charlie", "Alice", 10.0));
    chain.add_transaction(Transaction::new("Roseanne", "Bob", 5.0));

    // Mine again
    chain.mine_pending_transactions("Roseanne");

    // Print chain
    chain.print_chain();

    // Check balances
    println!("\n💰 BALANCES:");
    for addr in ["Alice", "Bob", "Charlie", "Roseanne"] {
        println!("  {:<12}: {:.2} coins", addr, chain.get_balance(addr));
    }

    // Validate chain
    println!("\n🔍 VALIDATION:");
    if chain.is_valid() {
        println!("  ✅ Blockchain is valid!");
    } else {
        println!("  ❌ Blockchain is INVALID!");
    }

    // Tamper test
    println!("\n🔨 TAMPER TEST:");
    println!("  Tampering with block #1 data...");
    chain.chain[1].transactions[0].amount = 999999.0;
    if chain.is_valid() {
        println!("  Blockchain is valid (unexpected!)");
    } else {
        println!("  ✅ Tamper detected! Blockchain is invalid.");
    }
}
