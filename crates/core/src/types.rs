use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use uuid::Uuid;

/// Represents a Solana wallet being tracked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub address: Pubkey,
    pub label: Option<String>,
    pub first_seen: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub metrics: WalletMetrics,
    pub risk_score: f64,
    pub smart_money_score: f64,
    pub is_tracked: bool,
}

/// Performance metrics for a wallet
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalletMetrics {
    pub total_trades: u64,
    pub winning_trades: u64,
    pub losing_trades: u64,
    pub win_rate: f64,
    pub total_pnl: Decimal,
    pub total_pnl_percentage: f64,
    pub avg_hold_time_seconds: f64,
    pub avg_profit_per_trade: Decimal,
    pub largest_win: Decimal,
    pub largest_loss: Decimal,
    pub sharpe_ratio: Option<f64>,
    pub max_drawdown: f64,
    pub trades_last_24h: u64,
    pub trades_last_7d: u64,
    pub volume_24h: Decimal,
    pub volume_7d: Decimal,
}

/// Represents a token on Solana
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub mint: Pubkey,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub metadata: TokenMetadata,
    pub security: SecurityInfo,
    pub market_data: MarketData,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Token metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenMetadata {
    pub logo_url: Option<String>,
    pub website: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub description: Option<String>,
}

/// Security analysis for a token
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityInfo {
    pub is_scam: bool,
    pub is_bundle: bool,
    pub rugcheck_score: Option<f64>,
    pub lp_locked: bool,
    pub lp_lock_duration: Option<i64>,
    pub mint_authority_disabled: bool,
    pub freeze_authority_disabled: bool,
    pub top_holders_percentage: f64,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

impl Default for RiskLevel {
    fn default() -> Self {
        Self::Medium
    }
}

/// Market data for a token
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarketData {
    pub price_usd: Decimal,
    pub price_sol: Decimal,
    pub market_cap: Decimal,
    pub liquidity_usd: Decimal,
    pub volume_24h: Decimal,
    pub price_change_24h: f64,
    pub price_change_1h: f64,
    pub price_change_5m: f64,
    pub holders: Option<u64>,
    pub dex: Option<String>,
}

/// Represents a trade (buy or sell)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub wallet: Pubkey,
    pub token_mint: Pubkey,
    pub side: TradeSide,
    pub amount_in: Decimal,
    pub amount_out: Decimal,
    pub price_usd: Decimal,
    pub market_cap_at_trade: Decimal,
    pub signature: String,
    pub timestamp: DateTime<Utc>,
    pub block_time: i64,
    pub dex: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TradeSide {
    Buy,
    Sell,
}

/// Represents a completed trade pair (entry + exit)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradePosition {
    pub id: Uuid,
    pub wallet: Pubkey,
    pub token_mint: Pubkey,
    pub entry_trade: Trade,
    pub exit_trade: Option<Trade>,
    pub pnl: Option<Decimal>,
    pub pnl_percentage: Option<f64>,
    pub hold_time_seconds: Option<f64>,
    pub entry_market_cap: Decimal,
    pub exit_market_cap: Option<Decimal>,
    pub status: PositionStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PositionStatus {
    Open,
    Closed,
    PartiallyFilled,
}

/// Copy trade signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyTradeSignal {
    pub id: Uuid,
    pub source_wallet: Pubkey,
    pub token_mint: Pubkey,
    pub side: TradeSide,
    pub source_trade: Trade,
    pub confidence_score: f64,
    pub reasons: Vec<String>,
    pub recommended_size: Decimal,
    pub priority: SignalPriority,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SignalPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Our bot's portfolio state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub wallet_address: Pubkey,
    pub sol_balance: Decimal,
    pub positions: Vec<Position>,
    pub total_value_usd: Decimal,
    pub pnl_today: Decimal,
    pub pnl_week: Decimal,
    pub pnl_all_time: Decimal,
    pub updated_at: DateTime<Utc>,
}

/// A position in our portfolio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub token_mint: Pubkey,
    pub amount: Decimal,
    pub avg_entry_price: Decimal,
    pub current_price: Decimal,
    pub value_usd: Decimal,
    pub unrealized_pnl: Decimal,
    pub unrealized_pnl_percentage: f64,
    pub entry_time: DateTime<Utc>,
}

/// Trade execution order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeOrder {
    pub id: Uuid,
    pub token_mint: Pubkey,
    pub side: TradeSide,
    pub amount_in: Decimal,
    pub min_amount_out: Decimal,
    pub max_slippage_bps: u16,
    pub priority: OrderPriority,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Submitted,
    Confirmed,
    Failed,
    Cancelled,
}

/// Bundle detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleDetection {
    pub token_mint: Pubkey,
    pub is_bundle: bool,
    pub confidence: f64,
    pub related_wallets: Vec<Pubkey>,
    pub evidence: Vec<String>,
    pub detected_at: DateTime<Utc>,
}

/// Insider activity detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsiderActivity {
    pub wallet: Pubkey,
    pub token_mint: Pubkey,
    pub activity_type: InsiderActivityType,
    pub confidence: f64,
    pub correlated_wallets: Vec<Pubkey>,
    pub timing_score: f64,
    pub evidence: Vec<String>,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InsiderActivityType {
    EarlyAccumulation,
    CoordinatedBuying,
    PrePumpPositioning,
    InsiderSell,
    WhaleActivity,
}

/// Strategy mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StrategyMode {
    Scalping,     // seconds to minutes
    DayTrading,   // minutes to hours
    SwingTrading, // hours to days
}

/// Risk limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimits {
    pub max_position_size_usd: Decimal,
    pub max_position_size_percentage: f64,
    pub max_daily_loss_usd: Decimal,
    pub max_daily_loss_percentage: f64,
    pub max_slippage_bps: u16,
    pub min_liquidity_usd: Decimal,
    pub min_smart_money_score: f64,
    pub max_risk_score: f64,
    pub stop_loss_percentage: f64,
    pub take_profit_percentage: f64,
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            max_position_size_usd: Decimal::from(100),
            max_position_size_percentage: 10.0,
            max_daily_loss_usd: Decimal::from(50),
            max_daily_loss_percentage: 5.0,
            max_slippage_bps: 100, // 1%
            min_liquidity_usd: Decimal::from(10000),
            min_smart_money_score: 0.6,
            max_risk_score: 0.7,
            stop_loss_percentage: 15.0,
            take_profit_percentage: 50.0,
        }
    }
}

/// Wallet analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAnalysis {
    pub wallet: Pubkey,
    pub metrics: WalletMetrics,
    pub smart_money_score: f64,
    pub risk_score: f64,
    pub is_insider: bool,
    pub is_whale: bool,
    pub typical_hold_time: f64,
    pub best_entry_mc_range: (Decimal, Decimal),
    pub best_exit_mc_range: (Decimal, Decimal),
    pub preferred_tokens: Vec<Pubkey>,
    pub copy_traders_count: Option<u64>,
    pub analyzed_at: DateTime<Utc>,
}
