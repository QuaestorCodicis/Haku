use std::collections::HashMap;
use solana_sdk::pubkey::Pubkey;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct OpenPosition {
    pub token_mint: Pubkey,
    pub token_symbol: String,
    pub entry_time: DateTime<Utc>,
    pub entry_price: Decimal,
    pub entry_mc: Decimal,
    pub amount: Decimal,
    pub current_price: Decimal,
    pub current_mc: Decimal,
    pub unrealized_pnl: Decimal,
    pub unrealized_pnl_pct: f64,
    pub stop_loss: Decimal,
    pub take_profit: Decimal,
    pub hold_time_minutes: i64,
}

#[derive(Debug, Clone)]
pub struct ClosedTrade {
    pub token_mint: Pubkey,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DailyStats {
    pub total_trades: u32,
    pub wins: u32,
    pub losses: u32,
    pub win_rate: f64,
    pub total_pnl: Decimal,
    pub biggest_win: Decimal,
    pub biggest_loss: Decimal,
    pub avg_win: Decimal,
    pub avg_loss: Decimal,
    pub portfolio_value: Decimal,
    pub starting_value: Decimal,
}

impl DailyStats {
    pub fn get_latest(&self) -> &DailyStats {
        self
    }
}

pub struct PortfolioMonitor {
    pub(crate) positions: HashMap<Pubkey, OpenPosition>,
    closed_trades: Vec<ClosedTrade>,
    daily_stats: DailyStats,
}

impl PortfolioMonitor {
    pub fn new(starting_capital: Decimal) -> Self {
        Self {
            positions: HashMap::new(),
            closed_trades: vec![],
            daily_stats: DailyStats {
                starting_value: starting_capital,
                portfolio_value: starting_capital,
                ..Default::default()
            },
        }
    }

    pub fn get_daily_stats(&self) -> &DailyStats {
        &self.daily_stats
    }

    pub fn get_last_closed_trade(&self) -> Option<&ClosedTrade> {
        self.closed_trades.last()
    }

    /// Add new position
    pub fn open_position(&mut self, position: OpenPosition) {
        println!("\n┌─ NEW POSITION ────────────────────");
        println!("│ Token: {}", position.token_symbol);
        println!("│ Entry: ${:.6}", position.entry_price);
        println!("│ Amount: ${:.2}", position.amount);
        println!("│ Stop Loss: ${:.6} ({:.1}%)",
            position.stop_loss,
            ((position.stop_loss - position.entry_price) / position.entry_price * Decimal::from(100))
                .to_string().parse::<f64>().unwrap_or(0.0)
        );
        println!("│ Take Profit: ${:.6} ({:.1}%)",
            position.take_profit,
            ((position.take_profit - position.entry_price) / position.entry_price * Decimal::from(100))
                .to_string().parse::<f64>().unwrap_or(0.0)
        );
        println!("└───────────────────────────────────\n");

        self.positions.insert(position.token_mint, position);
    }

    /// Close position
    pub fn close_position(
        &mut self,
        token_mint: &Pubkey,
        exit_price: Decimal,
    ) -> Option<ClosedTrade> {
        if let Some(position) = self.positions.remove(token_mint) {
            let exit_time = Utc::now();
            let pnl = (exit_price - position.entry_price) * position.amount / position.entry_price;
            let pnl_pct = ((exit_price - position.entry_price) / position.entry_price * Decimal::from(100))
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0);
            let hold_time = (exit_time - position.entry_time).num_minutes();
            let is_win = pnl > Decimal::ZERO;

            let trade = ClosedTrade {
                token_mint: *token_mint,
                token_symbol: position.token_symbol.clone(),
                entry_time: position.entry_time,
                exit_time,
                entry_price: position.entry_price,
                exit_price,
                pnl,
                pnl_pct,
                hold_time_minutes: hold_time,
                is_win,
            };

            // Print close notification
            self.print_trade_closed(&trade);

            // Update stats
            self.daily_stats.total_trades += 1;
            if is_win {
                self.daily_stats.wins += 1;
            } else {
                self.daily_stats.losses += 1;
            }
            self.daily_stats.total_pnl += pnl;
            self.daily_stats.portfolio_value += pnl;

            if is_win && pnl > self.daily_stats.biggest_win {
                self.daily_stats.biggest_win = pnl;
            }
            if !is_win && pnl < self.daily_stats.biggest_loss {
                self.daily_stats.biggest_loss = pnl;
            }

            self.daily_stats.win_rate = if self.daily_stats.total_trades > 0 {
                (self.daily_stats.wins as f64 / self.daily_stats.total_trades as f64) * 100.0
            } else {
                0.0
            };

            self.closed_trades.push(trade.clone());

            Some(trade)
        } else {
            None
        }
    }

    /// Update open positions with current prices
    pub fn update_prices(&mut self, prices: &HashMap<Pubkey, (Decimal, Decimal)>) {
        for (token_mint, position) in self.positions.iter_mut() {
            if let Some((current_price, current_mc)) = prices.get(token_mint) {
                position.current_price = *current_price;
                position.current_mc = *current_mc;
                position.unrealized_pnl = (*current_price - position.entry_price)
                    * position.amount / position.entry_price;
                position.unrealized_pnl_pct = ((*current_price - position.entry_price)
                    / position.entry_price * Decimal::from(100))
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0);
                position.hold_time_minutes = (Utc::now() - position.entry_time).num_minutes();
            }
        }
    }

    /// Display dashboard
    pub fn print_dashboard(&self) {
        println!("\n╔═══════════════════════════════════════════════════════════╗");
        println!("║                    PORTFOLIO DASHBOARD                     ║");
        println!("╠═══════════════════════════════════════════════════════════╣");
        println!("║ Portfolio Value: ${:.2}", self.daily_stats.portfolio_value);
        println!("║ Daily PnL: ${:.2} ({:.1}%)",
            self.daily_stats.total_pnl,
            if self.daily_stats.starting_value > Decimal::ZERO {
                ((self.daily_stats.total_pnl / self.daily_stats.starting_value) * Decimal::from(100))
                    .to_string().parse::<f64>().unwrap_or(0.0)
            } else { 0.0 }
        );
        println!("║ Win Rate: {}/{} ({:.1}%)",
            self.daily_stats.wins,
            self.daily_stats.total_trades,
            self.daily_stats.win_rate
        );
        println!("╠═══════════════════════════════════════════════════════════╣");

        // Open positions
        if self.positions.is_empty() {
            println!("║ No Open Positions");
        } else {
            println!("║ OPEN POSITIONS:");
            for position in self.positions.values() {
                let pnl_emoji = if position.unrealized_pnl_pct > 0.0 { "📈" } else { "📉" };
                println!("║  {} {} | {:.1}% | ${:.2}",
                    pnl_emoji,
                    position.token_symbol,
                    position.unrealized_pnl_pct,
                    position.unrealized_pnl
                );
            }
        }

        println!("╠═══════════════════════════════════════════════════════════╣");
        println!("║ Today's Stats:");
        println!("║  Biggest Win: ${:.2}", self.daily_stats.biggest_win);
        println!("║  Biggest Loss: ${:.2}", self.daily_stats.biggest_loss);
        println!("╚═══════════════════════════════════════════════════════════╝\n");
    }

    /// Print trade closed notification
    fn print_trade_closed(&self, trade: &ClosedTrade) {
        let emoji = if trade.is_win { "✅" } else { "❌" };
        let result = if trade.is_win { "WIN" } else { "LOSS" };

        println!("\n┌─ TRADE CLOSED ────────────────────");
        println!("│ {} {}", emoji, result);
        println!("│ Token: {}", trade.token_symbol);
        println!("│ Entry: ${:.6}", trade.entry_price);
        println!("│ Exit: ${:.6}", trade.exit_price);
        println!("│ PnL: ${:.2} ({:.1}%)", trade.pnl, trade.pnl_pct);
        println!("│ Hold Time: {} min", trade.hold_time_minutes);
        println!("└───────────────────────────────────\n");

        // Big win celebration
        if trade.pnl > Decimal::from(5) {
            println!("🎉 BIG WIN! ${:.2} profit! 🎉\n", trade.pnl);
        }
    }

    /// Print summary report
    pub fn print_summary(&self) {
        println!("\n╔═══════════════════════════════════════════════════════════╗");
        println!("║                      TRADING SUMMARY                       ║");
        println!("╠═══════════════════════════════════════════════════════════╣");
        println!("║ Total Trades: {}", self.closed_trades.len());
        println!("║ Wins: {} | Losses: {}", self.daily_stats.wins, self.daily_stats.losses);
        println!("║ Win Rate: {:.1}%", self.daily_stats.win_rate);
        println!("║ Total PnL: ${:.2}", self.daily_stats.total_pnl);
        println!("║ ROI: {:.1}%",
            if self.daily_stats.starting_value > Decimal::ZERO {
                ((self.daily_stats.total_pnl / self.daily_stats.starting_value) * Decimal::from(100))
                    .to_string().parse::<f64>().unwrap_or(0.0)
            } else { 0.0 }
        );
        println!("╠═══════════════════════════════════════════════════════════╣");
        println!("║ Recent Trades:");

        for trade in self.closed_trades.iter().rev().take(5) {
            let emoji = if trade.is_win { "✅" } else { "❌" };
            println!("║  {} {} | {:.1}% | ${:.2}",
                emoji,
                trade.token_symbol,
                trade.pnl_pct,
                trade.pnl
            );
        }

        println!("╚═══════════════════════════════════════════════════════════╝\n");
    }
}
