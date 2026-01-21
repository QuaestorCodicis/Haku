use std::collections::HashMap;
use solana_sdk::pubkey::Pubkey;
use rust_decimal::Decimal;
use chrono::Utc;
use tracing::{info, warn};
use trading_core::*;
use trading_data::*;
use trading_analysis::*;

use crate::portfolio_monitor::{OpenPosition, PortfolioMonitor};

pub struct PositionManager;

impl PositionManager {
    pub fn new() -> Self {
        Self
    }

    /// Check all open positions and close if stop-loss, take-profit, or chart signals indicate exit
    pub async fn check_and_update_positions(
        &self,
        portfolio: &mut PortfolioMonitor,
        token_fetcher: &TokenDataFetcher,
    ) -> anyhow::Result<()> {
        let positions: Vec<Pubkey> = portfolio.get_position_mints();

        if positions.is_empty() {
            return Ok(());
        }

        info!("📊 Checking {} open positions...", positions.len());

        let mut prices_to_update = HashMap::new();
        let mut positions_to_close = Vec::new();

        for token_mint in positions {
            // Fetch current token data
            match token_fetcher.get_token_data(&token_mint).await {
                Ok(token) => {
                    let current_price = token.market_data.price_usd;
                    let current_mc = token.market_data.market_cap;

                    prices_to_update.insert(token_mint, (current_price, current_mc));

                    // Get the position to check exit conditions
                    if let Some(position) = portfolio.get_position(&token_mint) {
                        let should_exit = self.should_exit_position(&position, &token);

                        if should_exit {
                            positions_to_close.push((token_mint, current_price));
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to fetch token data for {}: {}", token_mint, e);
                }
            }

            // Rate limiting
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        // Update all prices
        portfolio.update_prices(&prices_to_update);

        // Close positions that hit exit conditions
        for (token_mint, exit_price) in positions_to_close {
            portfolio.close_position(&token_mint, exit_price);
        }

        Ok(())
    }

    /// Determine if position should be exited
    fn should_exit_position(&self, position: &OpenPosition, token: &Token) -> bool {
        let current_price = token.market_data.price_usd;

        // 1. Stop-loss hit
        if current_price <= position.stop_loss {
            info!("🛑 Stop-loss triggered for {}: ${} <= ${}",
                position.token_symbol, current_price, position.stop_loss);
            return true;
        }

        // 2. Take-profit hit
        if current_price >= position.take_profit {
            info!("🎯 Take-profit triggered for {}: ${} >= ${}",
                position.token_symbol, current_price, position.take_profit);
            return true;
        }

        // 3. Chart shows strong sell signal
        let chart_signal = ChartAnalyzer::analyze_entry_exit(token);
        match chart_signal.action {
            TradeAction::StrongSell => {
                info!("📉 Strong sell signal for {}: {}",
                    position.token_symbol, chart_signal.reason);
                return true;
            }
            TradeAction::Sell if position.unrealized_pnl_pct > 15.0 => {
                // Only exit on weak sell if we're already up 15%+
                info!("📊 Sell signal with profit for {}: {} (+{:.1}%)",
                    position.token_symbol, chart_signal.reason, position.unrealized_pnl_pct);
                return true;
            }
            _ => {}
        }

        // 4. Time-based exit (position open too long)
        let hold_time_hours = position.hold_time_minutes / 60;
        if hold_time_hours > 24 && position.unrealized_pnl_pct < 5.0 {
            info!("⏰ Time exit for {} after {} hours with low profit ({:.1}%)",
                position.token_symbol, hold_time_hours, position.unrealized_pnl_pct);
            return true;
        }

        // 5. Trailing stop (moved stop-loss up as price rises)
        // If we're up 30%+, exit if price drops 15% from peak
        if position.unrealized_pnl_pct > 30.0 {
            let trailing_stop = position.take_profit * Decimal::from_f64_retain(0.85).unwrap();
            if current_price <= trailing_stop {
                info!("📈 Trailing stop triggered for {}: ${} <= ${}",
                    position.token_symbol, current_price, trailing_stop);
                return true;
            }
        }

        false
    }
}

// Extension methods for PortfolioMonitor
impl PortfolioMonitor {
    pub fn get_position_mints(&self) -> Vec<Pubkey> {
        self.positions.keys().copied().collect()
    }

    pub fn get_position(&self, token_mint: &Pubkey) -> Option<&OpenPosition> {
        self.positions.get(token_mint)
    }
}
