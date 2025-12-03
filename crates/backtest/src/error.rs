use alphafield_core::QuantError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BacktestError {
    #[error("Core error: {0}")]
    Core(#[from] QuantError),

    #[error("Data error: {0}")]
    Data(String),

    #[error("Strategy error: {0}")]
    Strategy(String),

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: f64, available: f64 },

    #[error("Position not found: {0}")]
    PositionNotFound(String),
}

pub type Result<T> = std::result::Result<T, BacktestError>;
