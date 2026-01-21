pub mod schema;
pub mod trades;
pub mod wallets;
pub mod statistics;

use anyhow::Result;
use sqlx::sqlite::SqlitePool;
use tracing::info;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("Connecting to database: {}", database_url);

        let pool = SqlitePool::connect(database_url).await?;

        info!("Running migrations...");
        Self::run_migrations(&pool).await?;

        info!("Database initialized successfully");

        Ok(Self { pool })
    }

    /// Run database migrations
    async fn run_migrations(pool: &SqlitePool) -> Result<()> {
        sqlx::query(schema::CREATE_TRADES_TABLE)
            .execute(pool)
            .await?;

        sqlx::query(schema::CREATE_POSITIONS_TABLE)
            .execute(pool)
            .await?;

        sqlx::query(schema::CREATE_WALLETS_TABLE)
            .execute(pool)
            .await?;

        sqlx::query(schema::CREATE_WALLET_METRICS_TABLE)
            .execute(pool)
            .await?;

        sqlx::query(schema::CREATE_DAILY_STATS_TABLE)
            .execute(pool)
            .await?;

        sqlx::query(schema::CREATE_SIGNALS_TABLE)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Get a connection from the pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Get trade repository
    pub fn trades(&self) -> trades::TradeRepository {
        trades::TradeRepository::new(self.pool.clone())
    }

    /// Get wallet repository
    pub fn wallets(&self) -> wallets::WalletRepository {
        wallets::WalletRepository::new(self.pool.clone())
    }

    /// Get statistics repository
    pub fn statistics(&self) -> statistics::StatisticsRepository {
        statistics::StatisticsRepository::new(self.pool.clone())
    }
}
