use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::SqlitePool;
use std::str::FromStr;

#[derive(Clone)]
pub struct StatisticsRepository {
    pool: SqlitePool,
}

#[derive(Debug, Clone)]
pub struct DailyStatsRecord {
    pub date: NaiveDate,
    pub total_trades: i64,
    pub wins: i64,
    pub losses: i64,
    pub win_rate: f64,
    pub total_pnl: Decimal,
    pub biggest_win: Decimal,
    pub biggest_loss: Decimal,
    pub avg_win: Decimal,
    pub avg_loss: Decimal,
    pub portfolio_value: Decimal,
}

impl StatisticsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Update daily statistics
    pub async fn update_daily_stats(
        &self,
        date: NaiveDate,
        total_trades: i64,
        wins: i64,
        losses: i64,
        win_rate: f64,
        total_pnl: Decimal,
        biggest_win: Decimal,
        biggest_loss: Decimal,
        avg_win: Decimal,
        avg_loss: Decimal,
        portfolio_value: Decimal,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO daily_stats (
                date, total_trades, wins, losses, win_rate, total_pnl,
                biggest_win, biggest_loss, avg_win, avg_loss, portfolio_value
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(date) DO UPDATE SET
                total_trades = excluded.total_trades,
                wins = excluded.wins,
                losses = excluded.losses,
                win_rate = excluded.win_rate,
                total_pnl = excluded.total_pnl,
                biggest_win = excluded.biggest_win,
                biggest_loss = excluded.biggest_loss,
                avg_win = excluded.avg_win,
                avg_loss = excluded.avg_loss,
                portfolio_value = excluded.portfolio_value,
                updated_at = datetime('now')
            "#,
        )
        .bind(date.format("%Y-%m-%d").to_string())
        .bind(total_trades)
        .bind(wins)
        .bind(losses)
        .bind(win_rate)
        .bind(total_pnl.to_string())
        .bind(biggest_win.to_string())
        .bind(biggest_loss.to_string())
        .bind(avg_win.to_string())
        .bind(avg_loss.to_string())
        .bind(portfolio_value.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get daily stats for a date range
    pub async fn get_stats_range(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<DailyStatsRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT date, total_trades, wins, losses, win_rate, total_pnl,
                   biggest_win, biggest_loss, avg_win, avg_loss, portfolio_value
            FROM daily_stats
            WHERE date >= ? AND date <= ?
            ORDER BY date DESC
            "#,
        )
        .bind(start_date.format("%Y-%m-%d").to_string())
        .bind(end_date.format("%Y-%m-%d").to_string())
        .fetch_all(&self.pool)
        .await?;

        let mut stats = Vec::new();
        for row in rows {
            let date_str: String = row.try_get("date")?;
            let total_pnl_str: String = row.try_get("total_pnl")?;
            let biggest_win_str: String = row.try_get("biggest_win")?;
            let biggest_loss_str: String = row.try_get("biggest_loss")?;
            let avg_win_str: String = row.try_get("avg_win")?;
            let avg_loss_str: String = row.try_get("avg_loss")?;
            let portfolio_value_str: String = row.try_get("portfolio_value")?;

            stats.push(DailyStatsRecord {
                date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")?,
                total_trades: row.try_get("total_trades")?,
                wins: row.try_get("wins")?,
                losses: row.try_get("losses")?,
                win_rate: row.try_get("win_rate")?,
                total_pnl: Decimal::from_str(&total_pnl_str)?,
                biggest_win: Decimal::from_str(&biggest_win_str)?,
                biggest_loss: Decimal::from_str(&biggest_loss_str)?,
                avg_win: Decimal::from_str(&avg_win_str)?,
                avg_loss: Decimal::from_str(&avg_loss_str)?,
                portfolio_value: Decimal::from_str(&portfolio_value_str)?,
            });
        }

        Ok(stats)
    }

    /// Get total statistics (all time)
    pub async fn get_total_stats(&self) -> Result<DailyStatsRecord> {
        let row = sqlx::query(
            r#"
            SELECT
                SUM(total_trades) as total_trades,
                SUM(wins) as wins,
                SUM(losses) as losses,
                AVG(win_rate) as win_rate,
                SUM(CAST(total_pnl AS REAL)) as total_pnl,
                MAX(CAST(biggest_win AS REAL)) as biggest_win,
                MIN(CAST(biggest_loss AS REAL)) as biggest_loss,
                AVG(CAST(avg_win AS REAL)) as avg_win,
                AVG(CAST(avg_loss AS REAL)) as avg_loss,
                MAX(CAST(portfolio_value AS REAL)) as portfolio_value
            FROM daily_stats
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DailyStatsRecord {
            date: Utc::now().date_naive(),
            total_trades: row.try_get::<Option<i64>, _>("total_trades")?.unwrap_or(0),
            wins: row.try_get::<Option<i64>, _>("wins")?.unwrap_or(0),
            losses: row.try_get::<Option<i64>, _>("losses")?.unwrap_or(0),
            win_rate: row.try_get::<Option<f64>, _>("win_rate")?.unwrap_or(0.0),
            total_pnl: Decimal::try_from(row.try_get::<Option<f64>, _>("total_pnl")?.unwrap_or(0.0))
                .unwrap_or(Decimal::ZERO),
            biggest_win: Decimal::try_from(
                row.try_get::<Option<f64>, _>("biggest_win")?
                    .unwrap_or(0.0),
            )
            .unwrap_or(Decimal::ZERO),
            biggest_loss: Decimal::try_from(
                row.try_get::<Option<f64>, _>("biggest_loss")?
                    .unwrap_or(0.0),
            )
            .unwrap_or(Decimal::ZERO),
            avg_win: Decimal::try_from(row.try_get::<Option<f64>, _>("avg_win")?.unwrap_or(0.0))
                .unwrap_or(Decimal::ZERO),
            avg_loss: Decimal::try_from(
                row.try_get::<Option<f64>, _>("avg_loss")?
                    .unwrap_or(0.0),
            )
            .unwrap_or(Decimal::ZERO),
            portfolio_value: Decimal::try_from(
                row.try_get::<Option<f64>, _>("portfolio_value")?
                    .unwrap_or(0.0),
            )
            .unwrap_or(Decimal::ZERO),
        })
    }

    /// Save signal to database
    pub async fn save_signal(
        &self,
        token_mint: &str,
        signal_type: &str,
        confidence: f64,
        smart_wallets_count: i32,
        avg_smart_score: f64,
        total_volume: Decimal,
        chart_action: Option<&str>,
        chart_confidence: Option<f64>,
        chart_reason: Option<&str>,
    ) -> Result<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO signals (
                token_mint, signal_type, confidence, smart_wallets_count,
                avg_smart_score, total_volume, chart_action, chart_confidence, chart_reason
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(token_mint)
        .bind(signal_type)
        .bind(confidence)
        .bind(smart_wallets_count)
        .bind(avg_smart_score)
        .bind(total_volume.to_string())
        .bind(chart_action)
        .bind(chart_confidence)
        .bind(chart_reason)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Mark signal as executed
    pub async fn mark_signal_executed(
        &self,
        signal_id: i64,
        execution_price: Decimal,
        execution_time: DateTime<Utc>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE signals
            SET executed = 1, execution_price = ?, execution_time = ?
            WHERE id = ?
            "#,
        )
        .bind(execution_price.to_string())
        .bind(execution_time.to_rfc3339())
        .bind(signal_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
