use crate::types::{RiskLimits, StrategyMode};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub solana: SolanaConfig,
    pub data_sources: DataSourcesConfig,
    pub database: DatabaseConfig,
    pub trading: TradingConfig,
    pub risk: RiskLimits,
    pub strategy: StrategyConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    /// Primary RPC endpoint (e.g., Helius free tier)
    pub rpc_url: String,
    /// Fallback RPC endpoints
    pub fallback_rpcs: Vec<String>,
    /// Wallet private key (base58 encoded)
    pub wallet_private_key: String,
    /// WebSocket URL for subscriptions
    pub ws_url: Option<String>,
    /// Commitment level
    pub commitment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourcesConfig {
    /// Jupiter API URL
    pub jupiter_api_url: String,
    /// DexScreener API URL
    pub dexscreener_api_url: String,
    /// Rugcheck API URL
    pub rugcheck_api_url: String,
    /// Birdeye API key (optional, can use free tier)
    pub birdeye_api_key: Option<String>,
    /// Helius API key (free tier)
    pub helius_api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub postgres_url: String,
    pub redis_url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    /// Enable/disable actual trading (false for dry-run mode)
    pub enabled: bool,
    /// Minimum time between trades (seconds)
    pub min_trade_interval: u64,
    /// Use Jito for MEV protection
    pub use_jito: bool,
    /// Jito tip amount in lamports
    pub jito_tip_lamports: u64,
    /// Default slippage tolerance (basis points)
    pub default_slippage_bps: u16,
    /// Priority fee in microlamports
    pub priority_fee_microlamports: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    /// Current strategy mode (can change dynamically)
    pub mode: StrategyMode,
    /// Enable adaptive mode switching
    pub adaptive_mode: bool,
    /// Minimum smart money score to copy trade
    pub min_smart_money_score: f64,
    /// Minimum wallet win rate to track (percentage)
    pub min_win_rate: f64,
    /// Minimum number of trades for wallet analysis
    pub min_trades_for_analysis: u64,
    /// Maximum wallets to track simultaneously
    pub max_tracked_wallets: usize,
    /// Insider detection sensitivity (0.0-1.0)
    pub insider_detection_threshold: f64,
    /// Bundle detection sensitivity (0.0-1.0)
    pub bundle_detection_threshold: f64,
    /// Token analysis update interval (seconds)
    pub token_analysis_interval: u64,
    /// Wallet analysis update interval (seconds)
    pub wallet_analysis_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable Telegram notifications
    pub telegram_enabled: bool,
    /// Telegram bot token
    pub telegram_bot_token: Option<String>,
    /// Telegram chat ID
    pub telegram_chat_id: Option<String>,
    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,
    /// Enable metrics export
    pub metrics_enabled: bool,
    /// Metrics export port
    pub metrics_port: u16,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            solana: SolanaConfig {
                rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
                fallback_rpcs: vec![
                    "https://solana-api.projectserum.com".to_string(),
                    "https://rpc.ankr.com/solana".to_string(),
                ],
                wallet_private_key: String::new(),
                ws_url: None,
                commitment: "confirmed".to_string(),
            },
            data_sources: DataSourcesConfig {
                jupiter_api_url: "https://quote-api.jup.ag/v6".to_string(),
                dexscreener_api_url: "https://api.dexscreener.com/latest".to_string(),
                rugcheck_api_url: "https://api.rugcheck.xyz/v1".to_string(),
                birdeye_api_key: None,
                helius_api_key: None,
            },
            database: DatabaseConfig {
                postgres_url: "postgresql://localhost/trading_bot".to_string(),
                redis_url: "redis://localhost:6379".to_string(),
                max_connections: 10,
            },
            trading: TradingConfig {
                enabled: false, // Start in dry-run mode
                min_trade_interval: 10,
                use_jito: true,
                jito_tip_lamports: 10000,
                default_slippage_bps: 100,
                priority_fee_microlamports: 10000,
            },
            risk: RiskLimits::default(),
            strategy: StrategyConfig {
                mode: StrategyMode::SwingTrading,
                adaptive_mode: true,
                min_smart_money_score: 0.7,
                min_win_rate: 60.0,
                min_trades_for_analysis: 10,
                max_tracked_wallets: 100,
                insider_detection_threshold: 0.75,
                bundle_detection_threshold: 0.8,
                token_analysis_interval: 60,
                wallet_analysis_interval: 300,
            },
            monitoring: MonitoringConfig {
                telegram_enabled: false,
                telegram_bot_token: None,
                telegram_chat_id: None,
                log_level: "info".to_string(),
                metrics_enabled: true,
                metrics_port: 9090,
            },
        }
    }
}
