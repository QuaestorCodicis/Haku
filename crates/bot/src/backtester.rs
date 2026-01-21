use anyhow::Result;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

use crate::persistence::{SerializableClosedTrade, TradeHistory};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub starting_capital: Decimal,
    pub position_size: Decimal,
    pub max_positions: usize,
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResults {
    pub starting_capital: Decimal,
    pub ending_capital: Decimal,
    pub total_pnl: Decimal,
    pub roi_pct: f64,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate_pct: f64,
    pub avg_win: Decimal,
    pub avg_loss: Decimal,
    pub biggest_win: Decimal,
    pub biggest_loss: Decimal,
    pub max_drawdown_pct: f64,
    pub profit_factor: f64,
    pub sharpe_ratio: f64,
    pub avg_hold_time_minutes: i64,
    pub trades: Vec<BacktestTrade>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestTrade {
    pub token_symbol: String,
    pub entry_time: DateTime<Utc>,
    pub exit_time: DateTime<Utc>,
    pub entry_price: Decimal,
    pub exit_price: Decimal,
    pub pnl: Decimal,
    pub pnl_pct: f64,
    pub hold_time_minutes: i64,
    pub is_win: bool,
}

pub struct Backtester {
    config: BacktestConfig,
}

impl Backtester {
    pub fn new(config: BacktestConfig) -> Self {
        Self { config }
    }

    /// Run backtest on historical trade data
    pub fn run(&self, trade_history: &TradeHistory) -> Result<BacktestResults> {
        info!("🔄 Starting backtest with {} historical trades", trade_history.closed_trades.len());

        if trade_history.closed_trades.is_empty() {
            return Err(anyhow::anyhow!("No historical trades to backtest"));
        }

        let mut capital = self.config.starting_capital;
        let mut peak_capital = capital;
        let mut max_drawdown = 0.0f64;

        let mut backtest_trades = Vec::new();
        let mut daily_returns = Vec::new();

        for trade in &trade_history.closed_trades {
            let backtest_trade = self.simulate_trade(trade, capital)?;

            capital += backtest_trade.pnl;

            // Track peak and drawdown
            if capital > peak_capital {
                peak_capital = capital;
            } else {
                let current_drawdown = ((peak_capital - capital) / peak_capital * Decimal::from(100))
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0);

                if current_drawdown > max_drawdown {
                    max_drawdown = current_drawdown;
                }
            }

            // Track daily returns for Sharpe ratio
            let return_pct = backtest_trade.pnl_pct;
            daily_returns.push(return_pct);

            backtest_trades.push(backtest_trade);
        }

        // Calculate metrics
        let total_pnl = capital - self.config.starting_capital;
        let roi_pct = if self.config.starting_capital > Decimal::ZERO {
            ((total_pnl / self.config.starting_capital) * Decimal::from(100))
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let winning_trades: Vec<_> = backtest_trades.iter().filter(|t| t.is_win).collect();
        let losing_trades: Vec<_> = backtest_trades.iter().filter(|t| !t.is_win).collect();

        let win_rate_pct = if !backtest_trades.is_empty() {
            (winning_trades.len() as f64 / backtest_trades.len() as f64) * 100.0
        } else {
            0.0
        };

        let avg_win = if !winning_trades.is_empty() {
            winning_trades.iter().map(|t| t.pnl).sum::<Decimal>() / Decimal::from(winning_trades.len())
        } else {
            Decimal::ZERO
        };

        let avg_loss = if !losing_trades.is_empty() {
            losing_trades.iter().map(|t| t.pnl).sum::<Decimal>() / Decimal::from(losing_trades.len())
        } else {
            Decimal::ZERO
        };

        let biggest_win = winning_trades.iter().map(|t| t.pnl).max().unwrap_or(Decimal::ZERO);
        let biggest_loss = losing_trades.iter().map(|t| t.pnl).min().unwrap_or(Decimal::ZERO);

        // Profit factor
        let gross_profit: Decimal = winning_trades.iter().map(|t| t.pnl).sum();
        let gross_loss: Decimal = losing_trades.iter().map(|t| t.pnl).sum();
        let profit_factor = if gross_loss < Decimal::ZERO {
            (gross_profit / gross_loss.abs())
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        // Sharpe ratio (simplified: assumes risk-free rate = 0)
        let sharpe_ratio = self.calculate_sharpe_ratio(&daily_returns);

        // Average hold time
        let avg_hold_time_minutes = if !backtest_trades.is_empty() {
            backtest_trades.iter().map(|t| t.hold_time_minutes).sum::<i64>() / backtest_trades.len() as i64
        } else {
            0
        };

        Ok(BacktestResults {
            starting_capital: self.config.starting_capital,
            ending_capital: capital,
            total_pnl,
            roi_pct,
            total_trades: backtest_trades.len(),
            winning_trades: winning_trades.len(),
            losing_trades: losing_trades.len(),
            win_rate_pct,
            avg_win,
            avg_loss,
            biggest_win,
            biggest_loss,
            max_drawdown_pct: max_drawdown,
            profit_factor,
            sharpe_ratio,
            avg_hold_time_minutes,
            trades: backtest_trades,
        })
    }

    fn simulate_trade(
        &self,
        trade: &SerializableClosedTrade,
        _current_capital: Decimal,
    ) -> Result<BacktestTrade> {
        let entry_price = Decimal::from_str_exact(&trade.entry_price)?;
        let exit_price = Decimal::from_str_exact(&trade.exit_price)?;

        // Use configured position size
        let position_size = self.config.position_size;

        // Calculate PnL
        let pnl = ((exit_price - entry_price) / entry_price) * position_size;
        let pnl_pct = trade.pnl_pct;

        Ok(BacktestTrade {
            token_symbol: trade.token_symbol.clone(),
            entry_time: trade.entry_time,
            exit_time: trade.exit_time,
            entry_price,
            exit_price,
            pnl,
            pnl_pct,
            hold_time_minutes: trade.hold_time_minutes,
            is_win: pnl > Decimal::ZERO,
        })
    }

    fn calculate_sharpe_ratio(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;

        if returns.len() < 2 {
            return 0.0;
        }

        // Calculate standard deviation
        let variance = returns
            .iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / (returns.len() - 1) as f64;

        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            return 0.0;
        }

        // Sharpe ratio (annualized, assuming ~365 trades per year)
        let annualized_return = mean_return * 365.0;
        let annualized_std_dev = std_dev * (365.0f64).sqrt();

        annualized_return / annualized_std_dev
    }
}

impl BacktestResults {
    /// Print detailed backtest report
    pub fn print_report(&self) {
        println!("\n╔═══════════════════════════════════════════════════════════╗");
        println!("║                    BACKTEST RESULTS                        ║");
        println!("╠═══════════════════════════════════════════════════════════╣");
        println!("║ 💰 PERFORMANCE");
        println!("║   Starting Capital: ${:.2}", self.starting_capital);
        println!("║   Ending Capital:   ${:.2}", self.ending_capital);
        println!("║   Total PnL:        ${:.2}", self.total_pnl);
        println!("║   ROI:              {:.2}%", self.roi_pct);
        println!("║");
        println!("║ 📊 TRADE STATISTICS");
        println!("║   Total Trades:     {}", self.total_trades);
        println!("║   Winning Trades:   {}", self.winning_trades);
        println!("║   Losing Trades:    {}", self.losing_trades);
        println!("║   Win Rate:         {:.1}%", self.win_rate_pct);
        println!("║");
        println!("║ 💵 WIN/LOSS ANALYSIS");
        println!("║   Average Win:      ${:.2}", self.avg_win);
        println!("║   Average Loss:     ${:.2}", self.avg_loss);
        println!("║   Biggest Win:      ${:.2}", self.biggest_win);
        println!("║   Biggest Loss:     ${:.2}", self.biggest_loss);
        println!("║   Profit Factor:    {:.2}", self.profit_factor);
        println!("║");
        println!("║ 📈 RISK METRICS");
        println!("║   Max Drawdown:     {:.2}%", self.max_drawdown_pct);
        println!("║   Sharpe Ratio:     {:.2}", self.sharpe_ratio);
        println!("║   Avg Hold Time:    {} min", self.avg_hold_time_minutes);
        println!("╠═══════════════════════════════════════════════════════════╣");

        // Rating
        let rating = self.get_strategy_rating();
        println!("║ 🎯 STRATEGY RATING: {}", rating);

        println!("╚═══════════════════════════════════════════════════════════╝\n");
    }

    fn get_strategy_rating(&self) -> &'static str {
        let mut score = 0;

        // Win rate > 60%
        if self.win_rate_pct > 60.0 {
            score += 1;
        }

        // ROI > 20%
        if self.roi_pct > 20.0 {
            score += 1;
        }

        // Profit factor > 1.5
        if self.profit_factor > 1.5 {
            score += 1;
        }

        // Sharpe ratio > 1.0
        if self.sharpe_ratio > 1.0 {
            score += 1;
        }

        // Max drawdown < 20%
        if self.max_drawdown_pct < 20.0 {
            score += 1;
        }

        match score {
            5 => "⭐⭐⭐⭐⭐ EXCELLENT",
            4 => "⭐⭐⭐⭐ GOOD",
            3 => "⭐⭐⭐ AVERAGE",
            2 => "⭐⭐ BELOW AVERAGE",
            _ => "⭐ NEEDS IMPROVEMENT",
        }
    }

    /// Save results to JSON file
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        info!("Saved backtest results to {}", path.display());
        Ok(())
    }
}
