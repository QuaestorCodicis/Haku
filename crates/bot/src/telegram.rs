use anyhow::Result;
use rust_decimal::Decimal;
use teloxide::prelude::*;
use teloxide::Bot;
use tracing::{error, info, warn};

use crate::portfolio_monitor::{ClosedTrade, DailyStats, OpenPosition};

pub struct TelegramNotifier {
    bot: Bot,
    chat_id: ChatId,
    enabled: bool,
}

impl TelegramNotifier {
    pub fn new(token: String, chat_id: i64) -> Self {
        let bot = Bot::new(token);
        info!("Telegram notifier initialized for chat ID: {}", chat_id);

        Self {
            bot,
            chat_id: ChatId(chat_id),
            enabled: true,
        }
    }

    pub fn disabled() -> Self {
        Self {
            bot: Bot::new("disabled"),
            chat_id: ChatId(0),
            enabled: false,
        }
    }

    /// Send a message to Telegram
    async fn send_message(&self, text: String) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        match self.bot.send_message(self.chat_id, text)
            .parse_mode(teloxide::types::ParseMode::Html)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to send Telegram message: {}", e);
                Err(e.into())
            }
        }
    }

    /// Notify when bot starts
    pub async fn notify_bot_started(&self, starting_capital: Decimal) {
        let message = format!(
            "🤖 <b>SOLANA TRADING BOT STARTED</b>\n\n\
             💰 Starting Capital: ${:.2}\n\
             📊 Mode: Paper Trading\n\
             ⏰ Time: {}\n\n\
             Ready to hunt alpha! 🚀",
            starting_capital,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        if let Err(e) = self.send_message(message).await {
            warn!("Failed to send startup notification: {}", e);
        }
    }

    /// Notify when a position is opened
    pub async fn notify_position_opened(&self, position: &OpenPosition, confidence: f64) {
        let message = format!(
            "🟢 <b>NEW POSITION OPENED</b>\n\n\
             🪙 Token: <code>{}</code>\n\
             💵 Entry Price: ${:.6}\n\
             💰 Amount: ${:.2}\n\
             📊 Confidence: {:.0}%\n\n\
             🎯 Take Profit: ${:.6} (+{:.1}%)\n\
             🛑 Stop Loss: ${:.6} ({:.1}%)\n\n\
             ⏰ {}",
            position.token_symbol,
            position.entry_price,
            position.amount,
            confidence * 100.0,
            position.take_profit,
            ((position.take_profit - position.entry_price) / position.entry_price * Decimal::from(100))
                .to_string().parse::<f64>().unwrap_or(0.0),
            position.stop_loss,
            ((position.stop_loss - position.entry_price) / position.entry_price * Decimal::from(100))
                .to_string().parse::<f64>().unwrap_or(0.0),
            chrono::Utc::now().format("%H:%M:%S UTC")
        );

        if let Err(e) = self.send_message(message).await {
            warn!("Failed to send position opened notification: {}", e);
        }
    }

    /// Notify when a position is closed
    pub async fn notify_position_closed(&self, trade: &ClosedTrade) {
        let emoji = if trade.is_win { "✅" } else { "❌" };
        let result = if trade.is_win { "WIN" } else { "LOSS" };
        let color = if trade.is_win { "🟢" } else { "🔴" };

        let message = format!(
            "{} <b>POSITION CLOSED - {}</b>\n\n\
             🪙 Token: <code>{}</code>\n\
             💵 Entry: ${:.6}\n\
             💵 Exit: ${:.6}\n\
             📊 PnL: ${:.2} ({:.1}%)\n\
             ⏱️ Hold Time: {} min\n\n\
             {} {}",
            color,
            result,
            trade.token_symbol,
            trade.entry_price,
            trade.exit_price,
            trade.pnl,
            trade.pnl_pct,
            trade.hold_time_minutes,
            emoji,
            if trade.is_win { "Nice trade!" } else { "Better luck next time" }
        );

        if let Err(e) = self.send_message(message).await {
            warn!("Failed to send position closed notification: {}", e);
        }

        // Big win celebration
        if trade.is_win && trade.pnl > Decimal::from(5) {
            self.notify_big_win(trade).await;
        }
    }

    /// Celebrate big wins
    async fn notify_big_win(&self, trade: &ClosedTrade) {
        let message = format!(
            "🎉🎉🎉 <b>BIG WIN!</b> 🎉🎉🎉\n\n\
             💰 Profit: ${:.2}\n\
             📈 Gain: {:.1}%\n\
             🪙 Token: {}\n\n\
             Keep crushing it! 🚀🚀🚀",
            trade.pnl,
            trade.pnl_pct,
            trade.token_symbol
        );

        if let Err(e) = self.send_message(message).await {
            warn!("Failed to send big win notification: {}", e);
        }
    }

    /// Send portfolio update
    pub async fn notify_portfolio_update(&self, stats: &DailyStats) {
        let roi = if stats.starting_value > Decimal::ZERO {
            ((stats.total_pnl / stats.starting_value) * Decimal::from(100))
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let emoji = if stats.total_pnl > Decimal::ZERO {
            "📈"
        } else if stats.total_pnl < Decimal::ZERO {
            "📉"
        } else {
            "➡️"
        };

        let message = format!(
            "{} <b>PORTFOLIO UPDATE</b>\n\n\
             💰 Portfolio Value: ${:.2}\n\
             📊 Daily PnL: ${:.2} ({:.1}%)\n\
             🎯 Win Rate: {}/{} ({:.1}%)\n\n\
             📈 Biggest Win: ${:.2}\n\
             📉 Biggest Loss: ${:.2}\n\n\
             💎 ROI: {:.1}%\n\
             ⏰ {}",
            emoji,
            stats.portfolio_value,
            stats.total_pnl,
            roi,
            stats.wins,
            stats.total_trades,
            stats.win_rate,
            stats.biggest_win,
            stats.biggest_loss,
            roi,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        if let Err(e) = self.send_message(message).await {
            warn!("Failed to send portfolio update: {}", e);
        }
    }

    /// Notify about ultra-high confidence signal detected
    pub async fn notify_ultra_signal(&self, token_mint: &str, confidence: f64, smart_wallets_count: usize) {
        let message = format!(
            "🔥 <b>ULTRA SIGNAL DETECTED</b> 🔥\n\n\
             🪙 Token: <code>{}</code>\n\
             📊 Confidence: {:.0}%\n\
             👥 Smart Wallets: {}\n\n\
             Analyzing for entry...",
            &token_mint[..16],
            confidence * 100.0,
            smart_wallets_count
        );

        if let Err(e) = self.send_message(message).await {
            warn!("Failed to send ultra signal notification: {}", e);
        }
    }

    /// Notify about scam detection
    pub async fn notify_scam_detected(&self, token_mint: &str) {
        let message = format!(
            "⚠️ <b>SCAM DETECTED</b>\n\n\
             🪙 Token: <code>{}</code>\n\n\
             ❌ Skipping this token for safety",
            &token_mint[..16]
        );

        if let Err(e) = self.send_message(message).await {
            warn!("Failed to send scam notification: {}", e);
        }
    }

    /// Notify about cycle completion
    pub async fn notify_cycle_complete(&self, cycle: u32, wallets_analyzed: usize, signals_found: usize) {
        let message = format!(
            "♻️ <b>Cycle #{} Complete</b>\n\n\
             👥 Wallets Analyzed: {}\n\
             🎯 Signals Found: {}\n\
             ⏰ {}",
            cycle,
            wallets_analyzed,
            signals_found,
            chrono::Utc::now().format("%H:%M:%S UTC")
        );

        if let Err(e) = self.send_message(message).await {
            warn!("Failed to send cycle complete notification: {}", e);
        }
    }

    /// Test notification to verify setup
    pub async fn test_notification(&self) -> Result<()> {
        let message = "✅ <b>Telegram Bot Connected!</b>\n\nYou will receive notifications here.".to_string();
        self.send_message(message).await
    }
}
