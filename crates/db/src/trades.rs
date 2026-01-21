use anyhow::Result;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use solana_sdk::pubkey::Pubkey;
use sqlx::SqlitePool;
use std::str::FromStr;
use trading_core::{Trade, TradeSide};
use uuid::Uuid;

#[derive(Clone)]
pub struct TradeRepository {
    pool: SqlitePool,
}

impl TradeRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Save a trade to the database
    pub async fn save_trade(&self, trade: &Trade) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO trades (
                id, wallet, token_mint, side, amount_in, amount_out,
                price_usd, market_cap_at_trade, signature, timestamp,
                block_time, dex
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(trade.id.to_string())
        .bind(trade.wallet.to_string())
        .bind(trade.token_mint.to_string())
        .bind(trade.side.to_string())
        .bind(trade.amount_in.to_string())
        .bind(trade.amount_out.to_string())
        .bind(trade.price_usd.to_string())
        .bind(trade.market_cap_at_trade.to_string())
        .bind(&trade.signature)
        .bind(trade.timestamp.to_rfc3339())
        .bind(trade.block_time)
        .bind(&trade.dex)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all trades for a wallet
    pub async fn get_wallet_trades(&self, wallet: &Pubkey, limit: i64) -> Result<Vec<Trade>> {
        let rows = sqlx::query(
            r#"
            SELECT id, wallet, token_mint, side, amount_in, amount_out,
                   price_usd, market_cap_at_trade, signature, timestamp,
                   block_time, dex
            FROM trades
            WHERE wallet = ?
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
        )
        .bind(wallet.to_string())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut trades = Vec::new();
        for row in rows {
            let id: String = row.try_get("id")?;
            let wallet: String = row.try_get("wallet")?;
            let token_mint: String = row.try_get("token_mint")?;
            let side: String = row.try_get("side")?;
            let amount_in: String = row.try_get("amount_in")?;
            let amount_out: String = row.try_get("amount_out")?;
            let price_usd: String = row.try_get("price_usd")?;
            let market_cap_at_trade: String = row.try_get("market_cap_at_trade")?;
            let signature: String = row.try_get("signature")?;
            let timestamp: String = row.try_get("timestamp")?;
            let block_time: i64 = row.try_get("block_time")?;
            let dex: String = row.try_get("dex")?;

            trades.push(Trade {
                id: Uuid::from_str(&id)?,
                wallet: Pubkey::from_str(&wallet)?,
                token_mint: Pubkey::from_str(&token_mint)?,
                side: match side.as_str() {
                    "Buy" => TradeSide::Buy,
                    "Sell" => TradeSide::Sell,
                    _ => continue,
                },
                amount_in: Decimal::from_str(&amount_in)?,
                amount_out: Decimal::from_str(&amount_out)?,
                price_usd: Decimal::from_str(&price_usd)?,
                market_cap_at_trade: Decimal::from_str(&market_cap_at_trade)?,
                signature,
                timestamp: DateTime::parse_from_rfc3339(&timestamp)?.with_timezone(&Utc),
                block_time,
                dex,
            });
        }

        Ok(trades)
    }

    /// Get recent trades (all wallets)
    pub async fn get_recent_trades(&self, limit: i64) -> Result<Vec<Trade>> {
        let rows = sqlx::query(
            r#"
            SELECT id, wallet, token_mint, side, amount_in, amount_out,
                   price_usd, market_cap_at_trade, signature, timestamp,
                   block_time, dex
            FROM trades
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut trades = Vec::new();
        for row in rows {
            let id: String = row.try_get("id")?;
            let wallet: String = row.try_get("wallet")?;
            let token_mint: String = row.try_get("token_mint")?;
            let side: String = row.try_get("side")?;
            let amount_in: String = row.try_get("amount_in")?;
            let amount_out: String = row.try_get("amount_out")?;
            let price_usd: String = row.try_get("price_usd")?;
            let market_cap_at_trade: String = row.try_get("market_cap_at_trade")?;
            let signature: String = row.try_get("signature")?;
            let timestamp: String = row.try_get("timestamp")?;
            let block_time: i64 = row.try_get("block_time")?;
            let dex: String = row.try_get("dex")?;

            trades.push(Trade {
                id: Uuid::from_str(&id)?,
                wallet: Pubkey::from_str(&wallet)?,
                token_mint: Pubkey::from_str(&token_mint)?,
                side: match side.as_str() {
                    "Buy" => TradeSide::Buy,
                    "Sell" => TradeSide::Sell,
                    _ => continue,
                },
                amount_in: Decimal::from_str(&amount_in)?,
                amount_out: Decimal::from_str(&amount_out)?,
                price_usd: Decimal::from_str(&price_usd)?,
                market_cap_at_trade: Decimal::from_str(&market_cap_at_trade)?,
                signature,
                timestamp: DateTime::parse_from_rfc3339(&timestamp)?.with_timezone(&Utc),
                block_time,
                dex,
            });
        }

        Ok(trades)
    }

    /// Save closed position to database
    pub async fn save_closed_position(
        &self,
        token_mint: &Pubkey,
        token_symbol: &str,
        entry_time: DateTime<Utc>,
        entry_price: Decimal,
        entry_mc: Decimal,
        amount: Decimal,
        stop_loss: Decimal,
        take_profit: Decimal,
        exit_time: DateTime<Utc>,
        exit_price: Decimal,
        exit_reason: &str,
        pnl: Decimal,
        pnl_pct: f64,
        hold_time_minutes: i64,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO positions (
                token_mint, token_symbol, entry_time, entry_price, entry_mc,
                amount, stop_loss, take_profit, status, exit_time, exit_price,
                exit_reason, pnl, pnl_pct, hold_time_minutes
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(token_mint.to_string())
        .bind(token_symbol)
        .bind(entry_time.to_rfc3339())
        .bind(entry_price.to_string())
        .bind(entry_mc.to_string())
        .bind(amount.to_string())
        .bind(stop_loss.to_string())
        .bind(take_profit.to_string())
        .bind("closed")
        .bind(exit_time.to_rfc3339())
        .bind(exit_price.to_string())
        .bind(exit_reason)
        .bind(pnl.to_string())
        .bind(pnl_pct)
        .bind(hold_time_minutes)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get trade count
    pub async fn get_trade_count(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM trades")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.try_get("count")?)
    }

    /// Get win rate
    pub async fn get_win_rate(&self) -> Result<f64> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(CASE WHEN pnl > 0 THEN 1 END) as wins,
                COUNT(*) as total
            FROM positions
            WHERE status = 'closed'
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let wins: i64 = row.try_get("wins")?;
        let total: i64 = row.try_get("total")?;

        if total == 0 {
            return Ok(0.0);
        }

        Ok((wins as f64 / total as f64) * 100.0)
    }
}
