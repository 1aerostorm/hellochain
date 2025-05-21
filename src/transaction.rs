use chrono::prelude::*;
use sha2::{Sha256, Digest};

/// Определяет типы транзакций, поддерживаемые блокчейном
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionType {
    /// Простая передача средств между адресами
    Transfer,
    /// Смарт-контракт с кодом в виде строки
    SmartContract(String),
    /// Хранение произвольных данных
    Data(Vec<u8>),
}

/// Представляет транзакцию в блокчейне
#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub transaction_type: TransactionType,
    pub sender: String,
    pub receiver: String,
    pub amount: f64,
    pub fee: f64,
    #[allow(dead_code)] // Помечаем как используемые, чтобы убрать предупреждение
    pub timestamp: i64,
    #[allow(dead_code)]
    pub signature: String,
}

impl Transaction {
    /// Создает новую транзакцию с указанными параметрами
    pub fn new(sender: String, receiver: String, amount: f64, transaction_type: TransactionType) -> Self {
        let timestamp = Utc::now().timestamp();
        let tx_data = format!("{}{}{}{:?}", sender, receiver, amount, timestamp);
        let id = calculate_hash(&tx_data);
        
        let signature = format!("sig_{}", calculate_hash(&format!("{}{}", id, timestamp)));
        
        let fee = match transaction_type {
            TransactionType::Transfer => 0.001 * amount,
            TransactionType::SmartContract(_) => 0.01 * amount + 0.5,
            TransactionType::Data(ref data) => 0.005 * amount + (data.len() as f64 * 0.0001),
        };
        
        Transaction {
            id,
            transaction_type,
            sender,
            receiver,
            amount,
            fee,
            timestamp,
            signature,
        }
    }
    
    /// Проверяет валидность транзакции (наличие отправителя, получателя и положительной суммы)
    pub fn is_valid(&self) -> bool {
        !self.sender.is_empty() && !self.receiver.is_empty() && self.amount > 0.0
    }
}

/// Вычисляет SHA-256 хеш для переданных данных
pub fn calculate_hash(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}