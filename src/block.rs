use crate::transaction::{Transaction, calculate_hash};
use chrono::prelude::*;
use std::fmt::{self, Debug, Formatter};

/// Представляет блок в блокчейне, содержащий транзакции и метаданные
#[derive(Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub merkle_root: String,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub difficulty: usize,
    pub validator: Option<String>,
}

impl Debug for Block {
    /// Форматирует блок для вывода в отладочном режиме
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Block[{}]: {} at: {}, with: {} transactions, nonce: {}, difficulty: {}",
               &self.index,
               &self.hash,
               &self.timestamp,
               &self.transactions.len(),
               &self.nonce,
               &self.difficulty,
        )
    }
}

impl Block {
    /// Создает новый блок с указанным индексом, транзакциями, предыдущим хешем и сложностью
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String, difficulty: usize) -> Self {
        let now = Utc::now();
        let merkle_root = Block::calculate_merkle_root(&transactions);
        
        let mut block = Block {
            index,
            timestamp: now.timestamp(),
            transactions,
            merkle_root,
            previous_hash,
            hash: String::new(),
            nonce: 0,
            difficulty,
            validator: None,
        };
        
        block.hash = block.calculate_hash();
        block
    }
    
    /// Вычисляет корень дерева Меркла для списка транзакций
    pub fn calculate_merkle_root(transactions: &[Transaction]) -> String {
        if transactions.is_empty() {
            return String::from("0");
        }
        
        let mut hashes: Vec<String> = transactions
            .iter()
            .map(|tx| calculate_hash(&format!("{}{}{}", tx.sender, tx.receiver, tx.amount)))
            .collect();
        
        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            
            for i in (0..hashes.len()).step_by(2) {
                if i + 1 < hashes.len() {
                    let combined = format!("{}{}", hashes[i], hashes[i + 1]);
                    next_level.push(calculate_hash(&combined));
                } else {
                    next_level.push(hashes[i].clone());
                }
            }
            
            hashes = next_level;
        }
        
        hashes[0].clone()
    }
    
    /// Вычисляет SHA-256 хеш блока на основе его метаданных
    pub fn calculate_hash(&self) -> String {
        let data = format!("{}{}{}{}{}{}", 
            self.index,
            self.timestamp,
            &self.merkle_root,
            self.previous_hash,
            self.nonce,
            self.difficulty
        );
        
        calculate_hash(&data)
    }
    
    /// Майнит блок с использованием алгоритма Proof of Work
    pub fn mine_block(&mut self) {
        let target = "0".repeat(self.difficulty);
        
        while &self.hash[..self.difficulty] != target {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
        
        println!("Block mined: {} (difficulty: {}, nonce: {})", self.hash, self.difficulty, self.nonce);
    }
    
    /// Валидирует блок с использованием алгоритма Proof of Stake
    pub fn validate_with_pos(&mut self, validator: String, stake_amount: f64) -> bool {
        use rand::{rngs::ThreadRng, Rng};
        
        let mut rng = ThreadRng::default();
        let validation_threshold = stake_amount / 1000.0;
        let random_value: f64 = rng.random();
        
        if random_value <= validation_threshold {
            self.validator = Some(validator);
            self.hash = self.calculate_hash();
            return true;
        }
        
        false
    }
}