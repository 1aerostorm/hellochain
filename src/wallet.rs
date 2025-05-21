use crate::errors::BlockchainError;

/// Представляет кошелек в блокчейне с адресом, балансом и историей транзакций
pub struct Wallet {
    pub address: String,
    pub balance: f64,
    pub staking_balance: f64,
    pub transaction_history: Vec<String>,
}

impl Wallet {
    /// Создает новый кошелек с указанным адресом и нулевыми балансами
    pub fn new(address: String) -> Self {
        Wallet {
            address,
            balance: 0.0,
            staking_balance: 0.0,
            transaction_history: Vec::new(),
        }
    }
    
    /// Переводит указанную сумму с основного баланса на стейкинг для PoS
    #[allow(dead_code)] // Помечаем как используемые
    pub fn stake(&mut self, amount: f64) -> Result<(), BlockchainError> {
        if amount > self.balance {
            return Err(BlockchainError::InsufficientBalance {
                required: amount,
                available: self.balance,
            });
        }
        
        self.balance -= amount;
        self.staking_balance += amount;
        Ok(())
    }
    
    /// Возвращает указанную сумму со стейкинга на основной баланс
    #[allow(dead_code)]
    pub fn unstake(&mut self, amount: f64) -> Result<(), BlockchainError> {
        if amount > self.staking_balance {
            return Err(BlockchainError::InsufficientBalance {
                required: amount,
                available: self.staking_balance,
            });
        }
        
        self.staking_balance -= amount;
        self.balance += amount;
        Ok(())
    }
}