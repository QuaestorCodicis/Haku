use anyhow::Result;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use solana_sdk::pubkey::Pubkey;
use sqlx::SqlitePool;
use std::str::FromStr;
use trading_core::WalletMetrics;

#[derive(Clone)]
pub struct WalletRepository {
    pool: SqlitePool,
}

impl WalletRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Save or update wallet
    pub async fn save_wallet(
        &self,
        address: &Pubkey,
        label: Option<&str>,
        smart_money_score: f64,
        risk_score: f64,
        is_tracked: bool,
        first_seen: DateTime<Utc>,
        last_active: DateTime<Utc>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO wallets (address, label, smart_money_score, risk_score, is_tracked, first_seen, last_active)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(address) DO UPDATE SET
                label = excluded.label,
                smart_money_score = excluded.smart_money_score,
                risk_score = excluded.risk_score,
                is_tracked = excluded.is_tracked,
                last_active = excluded.last_active,
                updated_at = datetime('now')
            "#,
        )
        .bind(address.to_string())
        .bind(label)
        .bind(smart_money_score)
        .bind(risk_score)
        .bind(is_tracked as i32)
        .bind(first_seen.to_rfc3339())
        .bind(last_active.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Save wallet metrics snapshot
    pub async fn save_wallet_metrics(
        &self,
        wallet_address: &Pubkey,
        metrics: &WalletMetrics,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO wallet_metrics (
                wallet_address, total_trades, winning_trades, losing_trades,
                win_rate, total_pnl, total_pnl_percentage, avg_hold_time_seconds,
                avg_profit_per_trade, largest_win, largest_loss, sharpe_ratio,
                max_drawdown, trades_last_24h, trades_last_7d, volume_24h, volume_7d
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(wallet_address.to_string())
        .bind(metrics.total_trades as i64)
        .bind(metrics.winning_trades as i64)
        .bind(metrics.losing_trades as i64)
        .bind(metrics.win_rate)
        .bind(metrics.total_pnl.to_string())
        .bind(metrics.total_pnl_percentage)
        .bind(metrics.avg_hold_time_seconds)
        .bind(metrics.avg_profit_per_trade.to_string())
        .bind(metrics.largest_win.to_string())
        .bind(metrics.largest_loss.to_string())
        .bind(metrics.sharpe_ratio)
        .bind(metrics.max_drawdown)
        .bind(metrics.trades_last_24h as i64)
        .bind(metrics.trades_last_7d as i64)
        .bind(metrics.volume_24h.to_string())
        .bind(metrics.volume_7d.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get tracked wallets
    pub async fn get_tracked_wallets(&self) -> Result<Vec<Pubkey>> {
        let rows = sqlx::query(
            r#"
            SELECT address FROM wallets
            WHERE is_tracked = 1
            ORDER BY smart_money_score DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut wallets = Vec::new();
        for row in rows {
            let address: String = row.try_get("address")?;
            if let Ok(pubkey) = Pubkey::from_str(&address) {
                wallets.push(pubkey);
            }
        }

        Ok(wallets)
    }

    /// Get wallet performance history
    pub async fn get_wallet_history(&self, address: &Pubkey, days: i64) -> Result<Vec<WalletMetrics>> {
        let cutoff = Utc::now() - chrono::Duration::days(days);

        let rows = sqlx::query(
            r#"
            SELECT total_trades, winning_trades, losing_trades, win_rate,
                   total_pnl, total_pnl_percentage, avg_hold_time_seconds,
                   avg_profit_per_trade, largest_win, largest_loss, sharpe_ratio,
                   max_drawdown, trades_last_24h, trades_last_7d, volume_24h, volume_7d
            FROM wallet_metrics
            WHERE wallet_address = ? AND snapshot_time >= ?
            ORDER BY snapshot_time DESC
            "#,
        )
        .bind(address.to_string())
        .bind(cutoff.to_rfc3339())
        .fetch_all(&self.pool)
        .await?;

        let mut history = Vec::new();
        for row in rows {
            let total_pnl: String = row.try_get("total_pnl")?;
            let avg_profit_per_trade: String = row.try_get("avg_profit_per_trade")?;
            let largest_win: String = row.try_get("largest_win")?;
            let largest_loss: String = row.try_get("largest_loss")?;
            let volume_24h: String = row.try_get("volume_24h")?;
            let volume_7d: String = row.try_get("volume_7d")?;

            history.push(WalletMetrics {
                total_trades: row.try_get::<i64, _>("total_trades")? as u64,
                winning_trades: row.try_get::<i64, _>("winning_trades")? as u64,
                losing_trades: row.try_get::<i64, _>("losing_trades")? as u64,
                win_rate: row.try_get("win_rate")?,
                total_pnl: Decimal::from_str(&total_pnl)?,
                total_pnl_percentage: row.try_get("total_pnl_percentage")?,
                avg_hold_time_seconds: row.try_get("avg_hold_time_seconds")?,
                avg_profit_per_trade: Decimal::from_str(&avg_profit_per_trade)?,
                largest_win: Decimal::from_str(&largest_win)?,
                largest_loss: Decimal::from_str(&largest_loss)?,
                sharpe_ratio: row.try_get("sharpe_ratio")?,
                max_drawdown: row.try_get("max_drawdown")?,
                trades_last_24h: row.try_get::<i64, _>("trades_last_24h")? as u64,
                trades_last_7d: row.try_get::<i64, _>("trades_last_7d")? as u64,
                volume_24h: Decimal::from_str(&volume_24h)?,
                volume_7d: Decimal::from_str(&volume_7d)?,
            });
        }

        Ok(history)
    }
}
