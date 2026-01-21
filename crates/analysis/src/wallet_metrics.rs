use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use tracing::{debug, info};
use trading_core::{
    PositionStatus, Result, Trade, TradeSide, TradePosition, TradingError, WalletAnalysis,
    WalletMetrics,
};
use uuid::Uuid;

/// Wallet metrics calculator
pub struct WalletMetricsCalculator;

impl WalletMetricsCalculator {
    /// Calculate comprehensive wallet metrics from trade history
    pub fn calculate_metrics(trades: &[Trade]) -> Result<WalletMetrics> {
        if trades.is_empty() {
            return Ok(WalletMetrics::default());
        }

        // Group trades into positions (match buys with sells)
        let positions = Self::group_trades_into_positions(trades);

        let mut total_pnl = Decimal::ZERO;
        let mut winning_trades = 0;
        let mut losing_trades = 0;
        let mut total_hold_time_seconds = 0.0;
        let mut largest_win = Decimal::ZERO;
        let mut largest_loss = Decimal::ZERO;
        let mut pnl_values = Vec::new();

        for position in &positions {
            if let Some(pnl) = position.pnl {
                total_pnl += pnl;
                pnl_values.push(pnl);

                if pnl > Decimal::ZERO {
                    winning_trades += 1;
                    if pnl > largest_win {
                        largest_win = pnl;
                    }
                } else if pnl < Decimal::ZERO {
                    losing_trades += 1;
                    if pnl < largest_loss {
                        largest_loss = pnl;
                    }
                }
            }

            if let Some(hold_time) = position.hold_time_seconds {
                total_hold_time_seconds += hold_time;
            }
        }

        let closed_positions = positions
            .iter()
            .filter(|p| p.status == PositionStatus::Closed)
            .count() as u64;

        let total_trades = closed_positions;
        let win_rate = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        let avg_hold_time_seconds = if closed_positions > 0 {
            total_hold_time_seconds / closed_positions as f64
        } else {
            0.0
        };

        let avg_profit_per_trade = if total_trades > 0 {
            total_pnl / Decimal::from(total_trades)
        } else {
            Decimal::ZERO
        };

        // Calculate Sharpe ratio (simplified)
        let sharpe_ratio = if !pnl_values.is_empty() {
            Some(Self::calculate_sharpe_ratio(&pnl_values))
        } else {
            None
        };

        // Calculate max drawdown
        let max_drawdown = Self::calculate_max_drawdown(&pnl_values);

        // Calculate volume metrics
        let now = Utc::now();
        let (trades_24h, volume_24h) = Self::calculate_time_window_stats(trades, now, 24 * 3600);
        let (trades_7d, volume_7d) = Self::calculate_time_window_stats(trades, now, 7 * 24 * 3600);

        // Calculate total PnL percentage
        let total_pnl_percentage = if !pnl_values.is_empty() {
            // Approximate as average PnL percentage
            pnl_values
                .iter()
                .map(|&pnl| (pnl / Decimal::from(100)) * Decimal::from(100)) // Simplified
                .sum::<Decimal>()
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0)
                / pnl_values.len() as f64
        } else {
            0.0
        };

        Ok(WalletMetrics {
            total_trades,
            winning_trades,
            losing_trades,
            win_rate,
            total_pnl,
            total_pnl_percentage,
            avg_hold_time_seconds,
            avg_profit_per_trade,
            largest_win,
            largest_loss,
            sharpe_ratio,
            max_drawdown,
            trades_last_24h: trades_24h,
            trades_last_7d: trades_7d,
            volume_24h,
            volume_7d,
        })
    }

    /// Group trades into positions (match buys with sells)
    fn group_trades_into_positions(trades: &[Trade]) -> Vec<TradePosition> {
        let mut positions = Vec::new();
        let mut open_positions: HashMap<Pubkey, Vec<Trade>> = HashMap::new();

        // Sort trades by timestamp
        let mut sorted_trades = trades.to_vec();
        sorted_trades.sort_by_key(|t| t.timestamp);

        for trade in sorted_trades {
            match trade.side {
                TradeSide::Buy => {
                    // Open new position
                    open_positions
                        .entry(trade.token_mint)
                        .or_insert_with(Vec::new)
                        .push(trade);
                }
                TradeSide::Sell => {
                    // Close position
                    if let Some(buys) = open_positions.get_mut(&trade.token_mint) {
                        if let Some(buy_trade) = buys.pop() {
                            let hold_time = trade
                                .timestamp
                                .signed_duration_since(buy_trade.timestamp)
                                .num_seconds() as f64;

                            let pnl = trade.amount_out - buy_trade.amount_in;
                            let pnl_percentage = if buy_trade.amount_in > Decimal::ZERO {
                                ((pnl / buy_trade.amount_in) * Decimal::from(100))
                                    .to_string()
                                    .parse::<f64>()
                                    .unwrap_or(0.0)
                            } else {
                                0.0
                            };

                            positions.push(TradePosition {
                                id: Uuid::new_v4(),
                                wallet: trade.wallet,
                                token_mint: trade.token_mint,
                                entry_trade: buy_trade,
                                exit_trade: Some(trade),
                                pnl: Some(pnl),
                                pnl_percentage: Some(pnl_percentage),
                                hold_time_seconds: Some(hold_time),
                                entry_market_cap: Decimal::ZERO, // Filled later with market data
                                exit_market_cap: Some(Decimal::ZERO),
                                status: PositionStatus::Closed,
                            });
                        }
                    }
                }
            }
        }

        // Add remaining open positions
        for (token_mint, buys) in open_positions {
            for buy_trade in buys {
                positions.push(TradePosition {
                    id: Uuid::new_v4(),
                    wallet: buy_trade.wallet,
                    token_mint,
                    entry_trade: buy_trade,
                    exit_trade: None,
                    pnl: None,
                    pnl_percentage: None,
                    hold_time_seconds: None,
                    entry_market_cap: Decimal::ZERO,
                    exit_market_cap: None,
                    status: PositionStatus::Open,
                });
            }
        }

        positions
    }

    /// Calculate Sharpe ratio
    fn calculate_sharpe_ratio(pnl_values: &[Decimal]) -> f64 {
        if pnl_values.is_empty() {
            return 0.0;
        }

        let mean: f64 = pnl_values
            .iter()
            .map(|&v| v.to_string().parse::<f64>().unwrap_or(0.0))
            .sum::<f64>()
            / pnl_values.len() as f64;

        let variance: f64 = pnl_values
            .iter()
            .map(|&v| {
                let val = v.to_string().parse::<f64>().unwrap_or(0.0);
                (val - mean).powi(2)
            })
            .sum::<f64>()
            / pnl_values.len() as f64;

        let std_dev = variance.sqrt();

        if std_dev > 0.0 {
            mean / std_dev
        } else {
            0.0
        }
    }

    /// Calculate maximum drawdown
    fn calculate_max_drawdown(pnl_values: &[Decimal]) -> f64 {
        if pnl_values.is_empty() {
            return 0.0;
        }

        let mut cumulative_pnl = Decimal::ZERO;
        let mut peak = Decimal::ZERO;
        let mut max_dd = 0.0;

        for &pnl in pnl_values {
            cumulative_pnl += pnl;
            if cumulative_pnl > peak {
                peak = cumulative_pnl;
            }

            let drawdown = if peak > Decimal::ZERO {
                ((peak - cumulative_pnl) / peak * Decimal::from(100))
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0)
            } else {
                0.0
            };

            if drawdown > max_dd {
                max_dd = drawdown;
            }
        }

        max_dd
    }

    /// Calculate stats for a time window
    fn calculate_time_window_stats(
        trades: &[Trade],
        now: DateTime<Utc>,
        seconds: i64,
    ) -> (u64, Decimal) {
        let cutoff = now - chrono::Duration::seconds(seconds);

        let recent_trades: Vec<&Trade> = trades
            .iter()
            .filter(|t| t.timestamp >= cutoff)
            .collect();

        let count = recent_trades.len() as u64;
        let volume = recent_trades
            .iter()
            .map(|t| t.amount_in)
            .sum();

        (count, volume)
    }

    /// Analyze wallet for best entry/exit market caps
    pub fn analyze_entry_exit_patterns(positions: &[TradePosition]) -> (Decimal, Decimal, Decimal, Decimal) {
        let mut entry_mcs = Vec::new();
        let mut exit_mcs = Vec::new();

        for pos in positions {
            if pos.status == PositionStatus::Closed && pos.pnl.unwrap_or(Decimal::ZERO) > Decimal::ZERO {
                entry_mcs.push(pos.entry_market_cap);
                if let Some(exit_mc) = pos.exit_market_cap {
                    exit_mcs.push(exit_mc);
                }
            }
        }

        let entry_range = if !entry_mcs.is_empty() {
            let min = *entry_mcs.iter().min().unwrap();
            let max = *entry_mcs.iter().max().unwrap();
            (min, max)
        } else {
            (Decimal::ZERO, Decimal::ZERO)
        };

        let exit_range = if !exit_mcs.is_empty() {
            let min = *exit_mcs.iter().min().unwrap();
            let max = *exit_mcs.iter().max().unwrap();
            (min, max)
        } else {
            (Decimal::ZERO, Decimal::ZERO)
        };

        (entry_range.0, entry_range.1, exit_range.0, exit_range.1)
    }

    /// Build full wallet analysis
    pub fn build_wallet_analysis(
        wallet: &Pubkey,
        trades: &[Trade],
    ) -> Result<WalletAnalysis> {
        let metrics = Self::calculate_metrics(trades)?;
        let positions = Self::group_trades_into_positions(trades);

        // Calculate smart money score (simplified)
        let smart_money_score = Self::calculate_simple_smart_score(&metrics);

        // Calculate risk score
        let risk_score = Self::calculate_risk_score(&metrics);

        // Detect if wallet is likely insider
        let is_insider = Self::detect_insider_likelihood(&metrics, &positions);

        // Detect if wallet is whale (high volume)
        let is_whale = metrics.volume_7d > Decimal::from(100_000); // $100k+

        // Calculate typical hold time
        let typical_hold_time = metrics.avg_hold_time_seconds;

        // Get best entry/exit market cap ranges
        let (entry_min, entry_max, exit_min, exit_max) = Self::analyze_entry_exit_patterns(&positions);

        // Find preferred tokens
        let mut token_counts: HashMap<Pubkey, u32> = HashMap::new();
        for trade in trades {
            *token_counts.entry(trade.token_mint).or_insert(0) += 1;
        }
        let mut preferred_tokens: Vec<(Pubkey, u32)> = token_counts.into_iter().collect();
        preferred_tokens.sort_by(|a, b| b.1.cmp(&a.1));
        let preferred_tokens = preferred_tokens.into_iter().take(10).map(|(k, _)| k).collect();

        Ok(WalletAnalysis {
            wallet: *wallet,
            metrics,
            smart_money_score,
            risk_score,
            is_insider,
            is_whale,
            typical_hold_time,
            best_entry_mc_range: (entry_min, entry_max),
            best_exit_mc_range: (exit_min, exit_max),
            preferred_tokens,
            copy_traders_count: None, // Would need external API
            analyzed_at: Utc::now(),
        })
    }

    /// Calculate simple smart money score (0-1)
    fn calculate_simple_smart_score(metrics: &WalletMetrics) -> f64 {
        let mut score = 0.0;

        // Win rate (40% weight)
        score += (metrics.win_rate / 100.0) * 0.4;

        // Positive PnL (20% weight)
        if metrics.total_pnl > Decimal::ZERO {
            score += 0.2;
        }

        // Sharpe ratio (20% weight)
        if let Some(sharpe) = metrics.sharpe_ratio {
            score += (sharpe.abs().min(2.0) / 2.0) * 0.2;
        }

        // Low drawdown (10% weight)
        if metrics.max_drawdown < 20.0 {
            score += 0.1;
        }

        // Consistent activity (10% weight)
        if metrics.trades_last_7d >= 5 {
            score += 0.1;
        }

        score.min(1.0_f64)
    }

    /// Calculate risk score (0-1, higher is riskier)
    fn calculate_risk_score(metrics: &WalletMetrics) -> f64 {
        let mut risk: f64 = 0.0;

        // High drawdown
        if metrics.max_drawdown > 50.0 {
            risk += 0.3;
        } else if metrics.max_drawdown > 30.0 {
            risk += 0.2;
        }

        // Low win rate
        if metrics.win_rate < 40.0 {
            risk += 0.3;
        }

        // Negative PnL
        if metrics.total_pnl < Decimal::ZERO {
            risk += 0.2;
        }

        // Erratic trading
        if metrics.trades_last_24h > 50 {
            risk += 0.2;
        }

        risk.min(1.0_f64)
    }

    /// Detect if wallet shows insider patterns
    fn detect_insider_likelihood(metrics: &WalletMetrics, positions: &[TradePosition]) -> bool {
        // Criteria for insider:
        // 1. Very high win rate (>80%)
        // 2. Consistent large wins
        // 3. Trades executed at optimal times (just before pumps)

        if metrics.win_rate > 80.0 && metrics.total_trades >= 10 {
            return true;
        }

        if metrics.avg_profit_per_trade > Decimal::from(1000) {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_metrics_empty() {
        let metrics = WalletMetricsCalculator::calculate_metrics(&[]);
        assert!(metrics.is_ok());
        let m = metrics.unwrap();
        assert_eq!(m.total_trades, 0);
    }
}
