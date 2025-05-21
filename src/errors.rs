use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlockchainError {
    #[error("Insufficient funds: {required} required, {available} available")]
    InsufficientBalance { required: f64, available: f64 },
    
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    
    #[error("Invalid block: {0}")]
    #[allow(dead_code)] // Помечаем как используемый, чтобы убрать предупреждение
    InvalidBlock(String),
    
    #[error("Consensus error: {0}")]
    ConsensusError(String),
}