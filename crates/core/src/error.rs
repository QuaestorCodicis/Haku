use thiserror::Error;

#[derive(Error, Debug)]
pub enum TradingError {
    #[error("Solana RPC error: {0}")]
    RpcError(String),

    #[error("Data fetch error: {0}")]
    DataFetchError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Trading execution error: {0}")]
    ExecutionError(String),

    #[error("Insufficient funds: {0}")]
    InsufficientFunds(String),

    #[error("Risk limit exceeded: {0}")]
    RiskLimitExceeded(String),

    #[error("Token security risk: {0}")]
    SecurityRisk(String),

    #[error("Scam detected: {0}")]
    ScamDetected(String),

    #[error("Bundle detected: {0}")]
    BundleDetected(String),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, TradingError>;
