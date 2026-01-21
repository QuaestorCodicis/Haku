use anyhow::Result;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::fs;
use std::path::Path;
use tracing::{info, warn};

use crate::portfolio_monitor::{ClosedTrade, DailyStats};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeHistory {
    pub closed_trades: Vec<SerializableClosedTrade>,
    pub daily_stats: SerializableDailyStats,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableClosedTrade {
    pub token_mint: String,
    pub token_symbol: String,
    pub entry_time: DateTime<Utc>,
    pub exit_time: DateTime<Utc>,
    pub entry_price: String,
    pub exit_price: String,
    pub pnl: String,
    pub pnl_pct: f64,
    pub hold_time_minutes: i64,
    pub is_win: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableDailyStats {
    pub total_trades: u32,
    pub wins: u32,
    pub losses: u32,
    pub win_rate: f64,
    pub total_pnl: String,
    pub biggest_win: String,
    pub biggest_loss: String,
    pub portfolio_value: String,
    pub starting_value: String,
}

impl TradeHistory {
    pub fn new(starting_value: Decimal) -> Self {
        Self {
            closed_trades: Vec::new(),
            daily_stats: SerializableDailyStats {
                total_trades: 0,
                wins: 0,
                losses: 0,
                win_rate: 0.0,
                total_pnl: "0".to_string(),
                biggest_win: "0".to_string(),
                biggest_loss: "0".to_string(),
                portfolio_value: starting_value.to_string(),
                starting_value: starting_value.to_string(),
            },
            last_updated: Utc::now(),
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            warn!("Trade history file not found, creating new");
            return Ok(Self::new(Decimal::from(10))); // Default starting value
        }

        let json = fs::read_to_string(path)?;
        let history: TradeHistory = serde_json::from_str(&json)?;

        info!("Loaded {} closed trades from history", history.closed_trades.len());

        Ok(history)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;

        info!("Saved {} trades to {}", self.closed_trades.len(), path.display());

        Ok(())
    }

    pub fn add_closed_trade(&mut self, trade: &ClosedTrade) {
        self.closed_trades.push(SerializableClosedTrade {
            token_mint: trade.token_mint.to_string(),
            token_symbol: trade.token_symbol.clone(),
            entry_time: trade.entry_time,
            exit_time: trade.exit_time,
            entry_price: trade.entry_price.to_string(),
            exit_price: trade.exit_price.to_string(),
            pnl: trade.pnl.to_string(),
            pnl_pct: trade.pnl_pct,
            hold_time_minutes: trade.hold_time_minutes,
            is_win: trade.is_win,
        });

        self.last_updated = Utc::now();
    }

    pub fn update_daily_stats(&mut self, stats: &DailyStats) {
        self.daily_stats = SerializableDailyStats {
            total_trades: stats.total_trades,
            wins: stats.wins,
            losses: stats.losses,
            win_rate: stats.win_rate,
            total_pnl: stats.total_pnl.to_string(),
            biggest_win: stats.biggest_win.to_string(),
            biggest_loss: stats.biggest_loss.to_string(),
            portfolio_value: stats.portfolio_value.to_string(),
            starting_value: stats.starting_value.to_string(),
        };

        self.last_updated = Utc::now();
    }

    pub fn get_total_trades(&self) -> usize {
        self.closed_trades.len()
    }

    pub fn get_win_rate(&self) -> f64 {
        if self.closed_trades.is_empty() {
            return 0.0;
        }

        let wins = self.closed_trades.iter().filter(|t| t.is_win).count();
        (wins as f64 / self.closed_trades.len() as f64) * 100.0
    }

    pub fn get_total_pnl(&self) -> Decimal {
        self.closed_trades
            .iter()
            .filter_map(|t| Decimal::from_str_exact(&t.pnl).ok())
            .sum()
    }

    pub fn get_best_trades(&self, limit: usize) -> Vec<&SerializableClosedTrade> {
        let mut trades = self.closed_trades.iter().collect::<Vec<_>>();
        trades.sort_by(|a, b| {
            let a_pnl = Decimal::from_str_exact(&a.pnl).unwrap_or(Decimal::ZERO);
            let b_pnl = Decimal::from_str_exact(&b.pnl).unwrap_or(Decimal::ZERO);
            b_pnl.cmp(&a_pnl)
        });
        trades.into_iter().take(limit).collect()
    }

    pub fn get_worst_trades(&self, limit: usize) -> Vec<&SerializableClosedTrade> {
        let mut trades = self.closed_trades.iter().collect::<Vec<_>>();
        trades.sort_by(|a, b| {
            let a_pnl = Decimal::from_str_exact(&a.pnl).unwrap_or(Decimal::ZERO);
            let b_pnl = Decimal::from_str_exact(&b.pnl).unwrap_or(Decimal::ZERO);
            a_pnl.cmp(&b_pnl)
        });
        trades.into_iter().take(limit).collect()
    }

    pub fn print_summary(&self) {
        println!("\n╔═══════════════════════════════════════════════════════════╗");
        println!("║                  TRADE HISTORY SUMMARY                     ║");
        println!("╠═══════════════════════════════════════════════════════════╣");
        println!("║ Total Trades: {}", self.get_total_trades());
        println!("║ Win Rate: {:.1}%", self.get_win_rate());
        println!("║ Total PnL: ${:.2}", self.get_total_pnl());
        println!("╠═══════════════════════════════════════════════════════════╣");
        println!("║ Best Trades:");

        for (idx, trade) in self.get_best_trades(3).iter().enumerate() {
            println!("║  {}. {} | {:.1}% | ${}",
                idx + 1,
                trade.token_symbol,
                trade.pnl_pct,
                trade.pnl
            );
        }

        println!("╚═══════════════════════════════════════════════════════════╝\n");
    }
}
