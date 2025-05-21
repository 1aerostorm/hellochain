use crate::block::Block;
use crate::transaction::{Transaction, TransactionType, calculate_hash};
use crate::wallet::Wallet;
use crate::errors::BlockchainError;
use std::collections::HashMap;
use chrono::prelude::*;
use rand::{rngs::ThreadRng, Rng};

#[derive(Debug)]
pub enum ConsensusAlgorithm {
    ProofOfWork,
    ProofOfStake,
    #[allow(dead_code)] // TODO
    DelegatedProofOfStake,
}

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: usize,
    pub pending_transactions: Vec<Transaction>,
    pub mining_reward: f64,
    pub wallets: HashMap<String, Wallet>,
    pub consensus_algorithm: ConsensusAlgorithm,
    pub transaction_fees: f64,
    pub validators: HashMap<String, f64>,
}

impl Blockchain {
    /// Создает новый блокчейн с заданной сложностью, наградой за майнинг и алгоритмом консенсуса
    pub fn new(difficulty: usize, mining_reward: f64, consensus_algorithm: ConsensusAlgorithm) -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            difficulty,
            pending_transactions: Vec::new(),
            mining_reward,
            wallets: HashMap::new(),
            consensus_algorithm,
            transaction_fees: 0.0,
            validators: HashMap::new(),
        };
        
        blockchain.create_genesis_block();
        blockchain
    }
    
    /// Создает и добавляет генезис-блок (первый блок) в цепочку
    pub fn create_genesis_block(&mut self) {
        let genesis_block = Block::new(0, Vec::new(), String::from("0"), self.difficulty);
        self.chain.push(genesis_block);
        
        println!("Genesis block created");
    }
    
    /// Возвращает ссылку на последний блок в цепочке
    pub fn get_latest_block(&self) -> &Block {
        &self.chain[self.chain.len() - 1]
    }
    
    /// Создает новый кошелек с указанным адресом и возвращает ссылку на него
    pub fn create_wallet(&mut self, address: String) -> &Wallet {
        self.wallets.insert(address.clone(), Wallet::new(address.clone()));
        self.wallets.get(&address).unwrap()
    }
    
    /// Добавляет средства на кошелек по указанному адресу
    pub fn add_funds_to_wallet(&mut self, address: &str, amount: f64) -> Result<(), BlockchainError> {
        if let Some(wallet) = self.wallets.get_mut(address) {
            wallet.balance += amount;
            Ok(())
        } else {
            Err(BlockchainError::InvalidTransaction(format!("Кошелек {} не найден", address)))
        }
    }
    
    /// Добавляет транзакцию в список ожидающих с проверкой валидности и баланса
    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<(), BlockchainError> {
        if !transaction.is_valid() {
            return Err(BlockchainError::InvalidTransaction("Транзакция невалидна".to_string()));
        }
        
        let total_amount = transaction.amount + transaction.fee;
        
        if transaction.sender != "BLOCKCHAIN_REWARD" {
            if let Some(wallet) = self.wallets.get(&transaction.sender) {
                if wallet.balance < total_amount {
                    return Err(BlockchainError::InsufficientBalance {
                        required: total_amount,
                        available: wallet.balance,
                    });
                }
            } else {
                return Err(BlockchainError::InvalidTransaction(format!("Wallet recipient {} not found", transaction.sender)));
            }
            
            if let Some(wallet) = self.wallets.get_mut(&transaction.sender) {
                wallet.balance -= total_amount;
                wallet.transaction_history.push(transaction.id.clone());
            }
        }
        
        self.pending_transactions.push(transaction);
        Ok(())
    }
    
    /// Майнит ожидающие транзакции, создает новый блок и добавляет его в цепочку
    pub fn mine_pending_transactions(&mut self, miner_address: String) -> Result<(), BlockchainError> {
        if !self.wallets.contains_key(&miner_address) {
            return Err(BlockchainError::InvalidTransaction(format!("Miner wallet {} not found", miner_address)));
        }
        
        let total_fees: f64 = self.pending_transactions.iter().map(|tx| tx.fee).sum();
        self.transaction_fees = total_fees;
        
        let reward_tx = Transaction::new(
            String::from("BLOCKCHAIN_REWARD"),
            miner_address.clone(),
            self.mining_reward + total_fees,
            TransactionType::Transfer
        );
        
        self.pending_transactions.push(reward_tx);
        
        let mut new_block = Block::new(
            self.chain.len() as u64,
            self.pending_transactions.clone(),
            self.get_latest_block().hash.clone(),
            self.difficulty
        );
        
        match self.consensus_algorithm {
            ConsensusAlgorithm::ProofOfWork => {
                new_block.mine_block();
            },
            ConsensusAlgorithm::ProofOfStake => {
                if let Some(stake) = self.validators.get(&miner_address) {
                    if !new_block.validate_with_pos(miner_address.clone(), *stake) {
                        return Err(BlockchainError::ConsensusError("Cannot validate block with PoS".to_string()));
                    }
                } else {
                    return Err(BlockchainError::ConsensusError(format!("This address {} is not a validator", miner_address)));
                }
            },
            ConsensusAlgorithm::DelegatedProofOfStake => {
                let mut rng = ThreadRng::default();
                let is_delegate = rng.random_bool(0.5);
                
                if !is_delegate {
                    return Err(BlockchainError::ConsensusError("This address is not a delegate of this block".to_string()));
                }
                
                new_block.validator = Some(miner_address.clone());
            }
        }
        
        for tx in &new_block.transactions {
            if tx.sender != "BLOCKCHAIN_REWARD" && tx.receiver != "BLOCKCHAIN_REWARD" {
                if let Some(wallet) = self.wallets.get_mut(&tx.receiver) {
                    wallet.balance += tx.amount;
                    wallet.transaction_history.push(tx.id.clone());
                } else {
                    let mut new_wallet = Wallet::new(tx.receiver.clone());
                    new_wallet.balance = tx.amount;
                    new_wallet.transaction_history.push(tx.id.clone());
                    self.wallets.insert(tx.receiver.clone(), new_wallet);
                }
            }
        }
        
        self.chain.push(new_block);
        self.pending_transactions = Vec::new();
        self.transaction_fees = 0.0;
        
        Ok(())
    }
    
    /// Регистрирует валидатора для PoS с указанной суммой стейкинга
    pub fn add_validator(&mut self, address: String, stake_amount: f64) -> Result<(), BlockchainError> {
        if let Some(wallet) = self.wallets.get_mut(&address) {
            if wallet.balance < stake_amount {
                return Err(BlockchainError::InsufficientBalance {
                    required: stake_amount,
                    available: wallet.balance,
                });
            }
            
            wallet.balance -= stake_amount;
            wallet.staking_balance += stake_amount;
            self.validators.insert(address, stake_amount);
            Ok(())
        } else {
            Err(BlockchainError::InvalidTransaction(format!("Cannot find wallet {}", address)))
        }
    }
    
    /// Проверяет валидность всей цепочки блоков
    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];
            
            if current_block.hash != current_block.calculate_hash() {
                println!("Wrong hash of block # {}", i);
                return false;
            }
            
            if current_block.previous_hash != previous_block.hash {
                println!("Wrong previous block before block # {}", i);
                return false;
            }
            
            let merkle_root = Block::calculate_merkle_root(&current_block.transactions);
            if current_block.merkle_root != merkle_root {
                println!("Wrong Merkle root in block # {}", i);
                return false;
            }
        }
        
        true
    }
    
    /// Возвращает баланс кошелька по указанному адресу
    pub fn get_balance(&self, address: &str) -> f64 {
        if let Some(wallet) = self.wallets.get(address) {
            return wallet.balance;
        }
        
        0.0
    }
    
    /// Возвращает историю транзакций для указанного адреса
    #[allow(dead_code)]
    pub fn get_transaction_history(&self, address: &str) -> Vec<Transaction> {
        let mut history = Vec::new();
        
        for block in &self.chain {
            for tx in &block.transactions {
                if tx.sender == address || tx.receiver == address {
                    history.push(tx.clone());
                }
            }
        }
        
        history
    }
    
    /// Возвращает информацию о кошельке по указанному адресу
    pub fn get_wallet_info(&self, address: &str) -> Option<&Wallet> {
        self.wallets.get(address)
    }
    
    /// Ищет транзакцию по её ID
    #[allow(dead_code)]
    pub fn find_transaction(&self, tx_id: &str) -> Option<Transaction> {
        for block in &self.chain {
            for tx in &block.transactions {
                if tx.id == tx_id {
                    return Some(tx.clone());
                }
            }
        }
        
        None
    }
    
    /// Корректирует сложность майнинга на основе времени создания блоков
    pub fn adjust_difficulty(&mut self) {
        if self.chain.len() % 10 == 0 && self.chain.len() > 1 {
            let last_ten_blocks = &self.chain[self.chain.len() - 10..];
            let latest_block = self.get_latest_block();
            let first_of_last_ten = &last_ten_blocks[0];
            
            let time_diff = latest_block.timestamp - first_of_last_ten.timestamp;
            let avg_block_time = time_diff as f64 / 10.0;
            
            let target_time = 60.0;
            
            if avg_block_time < target_time * 0.9 {
                self.difficulty += 1;
                println!("Difficulty increased, current: {}", self.difficulty);
            } else if avg_block_time > target_time * 1.1 && self.difficulty > 1 {
                self.difficulty -= 1;
                println!("Difficulty decreased, current: {}", self.difficulty);
            }
        }
    }
    
    /// Создает смарт-контракт и добавляет его в виде транзакции
    pub fn create_smart_contract(&mut self, creator: String, code: String, initial_value: f64) -> Result<String, BlockchainError> {
        let contract_address = format!("contract_{}", calculate_hash(&format!("{}{}{}", creator, code, Utc::now().timestamp())));
        
        let tx = Transaction::new(
            creator,
            contract_address.clone(),
            initial_value,
            TransactionType::SmartContract(code)
        );
        
        self.add_transaction(tx)?;
        
        self.create_wallet(contract_address.clone());
        
        Ok(contract_address)
    }
    
    /// Сохраняет данные в блокчейне в виде транзакции
    pub fn store_data(&mut self, sender: String, data: Vec<u8>) -> Result<String, BlockchainError> {
        let data_id = format!("data_{}", calculate_hash(&format!("{}{:?}", sender, data)));
        
        let tx = Transaction::new(
            sender,
            String::from("BLOCKCHAIN_DATA"),
            0.1,
            TransactionType::Data(data)
        );
        
        self.add_transaction(tx)?;
        
        Ok(data_id)
    }
    
    /// Имитирует выполнение функции смарт-контракта
    pub fn execute_smart_contract(&mut self, contract_address: &str, function: &str, args: Vec<String>) -> Result<String, BlockchainError> {
        let mut contract_code = String::new();
        for block in &self.chain {
            for tx in &block.transactions {
                if let TransactionType::SmartContract(ref code) = tx.transaction_type {
                    if tx.receiver == contract_address {
                        contract_code = code.clone();
                        break;
                    }
                }
            }
            if !contract_code.is_empty() {
                break;
            }
        }
        
        if contract_code.is_empty() {
            return Err(BlockchainError::InvalidTransaction(format!("Smart contract {} not found", contract_address)));
        }
        
        Ok(format!("Called function {} in smart contract {}: {:?}", function, contract_address, args))
    }
}