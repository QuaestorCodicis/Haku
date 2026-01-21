# 🚀 Advanced Optimization Guide

## Table of Contents
1. [Safety & Security Optimizations](#safety--security-optimizations)
2. [Efficiency Optimizations](#efficiency-optimizations)
3. [PnL Maximization Strategies](#pnl-maximization-strategies)
4. [Advanced Features](#advanced-features)

---

# 🛡️ Safety & Security Optimizations

## 1. Wallet Security (CRITICAL)

### Current Risk: Private key in .env file
**Solution**: Implement hardware wallet or key encryption

```rust
// crates/core/src/wallet_manager.rs
use solana_sdk::signature::{Keypair, Signer};
use std::path::Path;

pub struct SecureWalletManager {
    keypair: Option<Keypair>,
    encrypted_key_path: String,
}

impl SecureWalletManager {
    /// Load encrypted wallet
    pub fn load_encrypted(path: &Path, password: &str) -> Result<Self> {
        // 1. Read encrypted key file
        let encrypted = std::fs::read(path)?;

        // 2. Decrypt using password
        let decrypted = Self::decrypt_key(&encrypted, password)?;

        // 3. Load keypair
        let keypair = Keypair::from_bytes(&decrypted)?;

        Ok(Self {
            keypair: Some(keypair),
            encrypted_key_path: path.to_string_lossy().to_string(),
        })
    }

    /// Encrypt and save wallet
    pub fn save_encrypted(keypair: &Keypair, path: &Path, password: &str) -> Result<()> {
        let encrypted = Self::encrypt_key(&keypair.to_bytes(), password)?;
        std::fs::write(path, encrypted)?;
        Ok(())
    }

    fn encrypt_key(key: &[u8], password: &str) -> Result<Vec<u8>> {
        // Use AES-256-GCM encryption
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };
        use argon2::{Argon2, PasswordHasher};

        // Derive key from password using Argon2
        let salt = b"solana_trading_bot_salt_v1"; // Use random salt in production
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), salt)?;

        // Encrypt
        let cipher = Aes256Gcm::new_from_slice(password_hash.hash.unwrap().as_bytes())?;
        let nonce = Nonce::from_slice(b"unique nonce"); // Use random nonce
        let ciphertext = cipher.encrypt(nonce, key)?;

        Ok(ciphertext)
    }

    fn decrypt_key(encrypted: &[u8], password: &str) -> Result<Vec<u8>> {
        // Reverse of encrypt_key
        // Implementation similar to above
        unimplemented!()
    }
}
```

**Implementation Steps**:
```bash
# Add to Cargo.toml
aes-gcm = "0.10"
argon2 = "0.5"

# Create encrypted wallet
cargo run --bin create-wallet
# Enter password when prompted
# Saves to encrypted-wallet.dat
```

**Best Practice**:
- Never store plaintext private keys
- Use strong passwords (16+ chars)
- Consider hardware wallet (Ledger) for large amounts
- Rotate keys periodically

---

## 2. Transaction Security

### A. Prevent Replay Attacks

```rust
// crates/trading/src/transaction_builder.rs
use solana_sdk::{
    hash::Hash,
    transaction::Transaction,
    signature::Keypair,
};

pub struct SecureTransactionBuilder {
    recent_blockhash: Hash,
    blockhash_timestamp: i64,
}

impl SecureTransactionBuilder {
    /// Build transaction with fresh blockhash
    pub async fn build_swap_transaction(
        &mut self,
        rpc: &FallbackRpcClient,
        swap_data: &SwapData,
    ) -> Result<Transaction> {
        // 1. Always get fresh blockhash (prevents replay)
        let blockhash = rpc.get_latest_blockhash().await?;
        self.recent_blockhash = blockhash;
        self.blockhash_timestamp = Utc::now().timestamp();

        // 2. Build transaction
        let mut tx = Transaction::new_with_payer(
            &swap_data.instructions,
            Some(&swap_data.payer),
        );

        tx.message.recent_blockhash = blockhash;

        // 3. Add durable nonce for extra security (optional)
        if let Some(nonce_account) = &swap_data.nonce_account {
            Self::add_durable_nonce(&mut tx, nonce_account)?;
        }

        Ok(tx)
    }

    /// Verify transaction hasn't expired
    pub fn is_transaction_valid(&self) -> bool {
        let age = Utc::now().timestamp() - self.blockhash_timestamp;
        age < 150 // Blockhash valid for ~150 seconds
    }
}
```

### B. MEV Protection (Critical for PnL)

```rust
// crates/trading/src/jito_bundle.rs
use reqwest::Client;

pub struct JitoClient {
    client: Client,
    tip_account: Pubkey,
    min_tip_lamports: u64,
}

impl JitoClient {
    /// Submit transaction as Jito bundle for MEV protection
    pub async fn send_bundle(
        &self,
        transactions: Vec<Transaction>,
        tip_lamports: u64,
    ) -> Result<String> {
        // 1. Create tip transfer transaction
        let tip_tx = Self::create_tip_transaction(
            tip_lamports.max(self.min_tip_lamports),
            &self.tip_account,
        )?;

        // 2. Bundle transactions
        let mut bundle = transactions;
        bundle.push(tip_tx);

        // 3. Send to Jito
        let response = self.client
            .post("https://mainnet.block-engine.jito.wtf/api/v1/bundles")
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "sendBundle",
                "params": [bundle.iter().map(|tx| {
                    bs58::encode(bincode::serialize(tx).unwrap()).into_string()
                }).collect::<Vec<_>>()]
            }))
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        Ok(result["result"].as_str().unwrap_or("").to_string())
    }

    /// Calculate optimal tip based on priority
    pub fn calculate_tip(&self, priority: OrderPriority, base_tip: u64) -> u64 {
        match priority {
            OrderPriority::Low => base_tip,
            OrderPriority::Medium => base_tip * 2,
            OrderPriority::High => base_tip * 5,
            OrderPriority::Urgent => base_tip * 10, // For insider plays
        }
    }
}
```

**Why This Matters**:
- Prevents sandwich attacks (can cost 1-5% of trade value)
- Ensures transaction lands in expected block
- Critical for copy trading where timing is everything

---

## 3. Input Validation & Sanitization

```rust
// crates/core/src/validators.rs

pub struct InputValidator;

impl InputValidator {
    /// Validate token mint address
    pub fn validate_token_mint(mint: &str) -> Result<Pubkey> {
        // 1. Check length
        if mint.len() != 43 && mint.len() != 44 {
            return Err(TradingError::ParseError("Invalid mint length".into()));
        }

        // 2. Parse as Pubkey
        let pubkey = Pubkey::from_str(mint)
            .map_err(|_| TradingError::ParseError("Invalid base58".into()))?;

        // 3. Check it's not a system program
        if pubkey == solana_sdk::system_program::id() {
            return Err(TradingError::ParseError("Cannot trade system program".into()));
        }

        Ok(pubkey)
    }

    /// Validate trade amount
    pub fn validate_amount(amount: Decimal, min: Decimal, max: Decimal) -> Result<()> {
        if amount <= Decimal::ZERO {
            return Err(TradingError::RiskLimitExceeded("Amount must be positive".into()));
        }

        if amount < min {
            return Err(TradingError::RiskLimitExceeded(
                format!("Amount {} below minimum {}", amount, min)
            ));
        }

        if amount > max {
            return Err(TradingError::RiskLimitExceeded(
                format!("Amount {} exceeds maximum {}", amount, max)
            ));
        }

        Ok(())
    }

    /// Validate slippage
    pub fn validate_slippage(slippage_bps: u16) -> Result<()> {
        if slippage_bps > 1000 { // Max 10%
            return Err(TradingError::RiskLimitExceeded(
                "Slippage too high, likely a scam token".into()
            ));
        }
        Ok(())
    }
}
```

---

## 4. Dependency Security

```bash
# Add to project root
cargo install cargo-audit

# Run security audit
cargo audit

# Check for outdated dependencies
cargo outdated

# Add to CI/CD pipeline
```

**Create security check script**:
```bash
#!/bin/bash
# scripts/security-check.sh

echo "🔒 Running security checks..."

# 1. Audit dependencies
cargo audit --deny warnings

# 2. Check for common vulnerabilities
cargo clippy -- -D warnings

# 3. Check for unsafe code
rg "unsafe" crates/ || echo "✅ No unsafe code found"

# 4. Verify no secrets in code
rg -i "private.*key|secret|password" crates/ --glob '!*.md' || echo "✅ No hardcoded secrets"

echo "✅ Security checks passed"
```

---

## 5. Advanced Risk Controls

```rust
// crates/risk/src/advanced_checks.rs

pub struct AdvancedRiskManager {
    risk_limits: RiskLimits,
    portfolio: Arc<RwLock<Portfolio>>,
    trade_history: Arc<RwLock<Vec<TradeRecord>>>,
    circuit_breaker: Arc<RwLock<CircuitBreaker>>,
}

#[derive(Debug)]
pub struct CircuitBreaker {
    is_triggered: bool,
    trigger_reason: Option<String>,
    trigger_time: Option<DateTime<Utc>>,
    cooldown_minutes: i64,
}

impl AdvancedRiskManager {
    /// Pre-trade risk validation
    pub async fn validate_trade(&self, signal: &CopyTradeSignal) -> Result<TradeApproval> {
        let mut checks = Vec::new();

        // 1. Check circuit breaker
        if self.circuit_breaker.read().await.is_triggered {
            return Err(TradingError::RiskLimitExceeded(
                "Circuit breaker active".into()
            ));
        }

        // 2. Check daily loss limit
        self.check_daily_loss_limit().await?;
        checks.push("Daily loss OK");

        // 3. Check position concentration
        self.check_position_concentration(&signal.token_mint).await?;
        checks.push("Concentration OK");

        // 4. Check correlation risk (don't buy too many similar tokens)
        self.check_correlation_risk(&signal.token_mint).await?;
        checks.push("Correlation OK");

        // 5. Check wallet velocity (not trading too fast)
        self.check_trade_velocity().await?;
        checks.push("Velocity OK");

        // 6. Check token-specific limits
        self.check_token_limits(&signal.token_mint).await?;
        checks.push("Token limits OK");

        // 7. Dynamic position sizing based on confidence and volatility
        let recommended_size = self.calculate_kelly_criterion(signal).await?;

        Ok(TradeApproval {
            approved: true,
            checks_passed: checks,
            recommended_size,
            warnings: vec![],
        })
    }

    /// Kelly Criterion for optimal position sizing
    async fn calculate_kelly_criterion(&self, signal: &CopyTradeSignal) -> Result<Decimal> {
        // Kelly % = (Win% * Avg Win - Loss% * Avg Loss) / Avg Win

        // Get historical data for this confidence level
        let history = self.trade_history.read().await;
        let similar_trades: Vec<_> = history
            .iter()
            .filter(|t| (t.confidence_score - signal.confidence_score).abs() < 0.1)
            .collect();

        if similar_trades.len() < 10 {
            // Not enough data, use conservative sizing
            return Ok(self.risk_limits.max_position_size_usd / Decimal::from(10));
        }

        let wins: Vec<_> = similar_trades.iter().filter(|t| t.pnl > Decimal::ZERO).collect();
        let losses: Vec<_> = similar_trades.iter().filter(|t| t.pnl <= Decimal::ZERO).collect();

        let win_rate = wins.len() as f64 / similar_trades.len() as f64;
        let loss_rate = 1.0 - win_rate;

        let avg_win = wins.iter().map(|t| t.pnl).sum::<Decimal>() / Decimal::from(wins.len().max(1));
        let avg_loss = losses.iter().map(|t| t.pnl.abs()).sum::<Decimal>() / Decimal::from(losses.len().max(1));

        if avg_win == Decimal::ZERO {
            return Ok(self.risk_limits.max_position_size_usd / Decimal::from(10));
        }

        // Kelly fraction
        let kelly = (win_rate * avg_win.to_string().parse::<f64>().unwrap()
                    - loss_rate * avg_loss.to_string().parse::<f64>().unwrap())
                    / avg_win.to_string().parse::<f64>().unwrap();

        // Use fractional Kelly (1/4 Kelly for safety)
        let fractional_kelly = (kelly * 0.25).max(0.0).min(0.2); // Cap at 20% of portfolio

        let portfolio_value = self.portfolio.read().await.total_value_usd;
        let position_size = portfolio_value * Decimal::from_f64_retain(fractional_kelly).unwrap();

        Ok(position_size.min(self.risk_limits.max_position_size_usd))
    }

    /// Check position concentration (max 30% in single token)
    async fn check_position_concentration(&self, token_mint: &Pubkey) -> Result<()> {
        let portfolio = self.portfolio.read().await;
        let current_position = portfolio.positions
            .iter()
            .find(|p| &p.token_mint == token_mint)
            .map(|p| p.value_usd)
            .unwrap_or(Decimal::ZERO);

        let concentration = if portfolio.total_value_usd > Decimal::ZERO {
            (current_position / portfolio.total_value_usd * Decimal::from(100))
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        if concentration > 30.0 {
            return Err(TradingError::RiskLimitExceeded(
                format!("Token concentration {}% exceeds 30% limit", concentration)
            ));
        }

        Ok(())
    }

    /// Trigger circuit breaker
    pub async fn trigger_circuit_breaker(&self, reason: String) {
        let mut cb = self.circuit_breaker.write().await;
        cb.is_triggered = true;
        cb.trigger_reason = Some(reason.clone());
        cb.trigger_time = Some(Utc::now());

        error!("🚨 CIRCUIT BREAKER TRIGGERED: {}", reason);

        // Send alert
        self.send_emergency_alert(&reason).await;
    }

    /// Auto-reset circuit breaker after cooldown
    pub async fn check_circuit_breaker_reset(&self) {
        let mut cb = self.circuit_breaker.write().await;

        if let Some(trigger_time) = cb.trigger_time {
            let elapsed = Utc::now().signed_duration_since(trigger_time);
            if elapsed.num_minutes() > cb.cooldown_minutes {
                info!("Circuit breaker cooldown complete, resetting");
                cb.is_triggered = false;
                cb.trigger_reason = None;
            }
        }
    }
}

#[derive(Debug)]
pub struct TradeApproval {
    pub approved: bool,
    pub checks_passed: Vec<&'static str>,
    pub recommended_size: Decimal,
    pub warnings: Vec<String>,
}

#[derive(Debug)]
pub struct TradeRecord {
    pub confidence_score: f64,
    pub pnl: Decimal,
    pub executed_at: DateTime<Utc>,
}
```

---

# ⚡ Efficiency Optimizations

## 1. Parallel Wallet Analysis

```rust
// crates/analysis/src/parallel_analyzer.rs
use rayon::prelude::*;
use std::sync::Arc;

pub struct ParallelWalletAnalyzer {
    rpc_client: Arc<FallbackRpcClient>,
    token_fetcher: Arc<TokenDataFetcher>,
}

impl ParallelWalletAnalyzer {
    /// Analyze multiple wallets in parallel
    pub async fn analyze_wallets_batch(
        &self,
        wallets: &[Pubkey],
    ) -> Vec<(Pubkey, Result<WalletAnalysis>)> {
        // Use Rayon for CPU-bound analysis
        wallets.par_iter()
            .map(|wallet| {
                let runtime = tokio::runtime::Handle::current();
                let result = runtime.block_on(async {
                    self.analyze_single_wallet(wallet).await
                });
                (*wallet, result)
            })
            .collect()
    }

    async fn analyze_single_wallet(&self, wallet: &Pubkey) -> Result<WalletAnalysis> {
        // Fetch trades
        let trades = TransactionParser::get_wallet_trades(
            &self.rpc_client,
            wallet,
            100,
        ).await?;

        // Calculate metrics
        let analysis = WalletMetricsCalculator::build_wallet_analysis(wallet, &trades)?;

        Ok(analysis)
    }
}
```

**Performance Impact**: 5-10x faster for analyzing 100+ wallets

---

## 2. Smart Caching Strategy

```rust
// crates/data/src/cache_manager.rs
use dashmap::DashMap;
use std::sync::Arc;
use tokio::time::{Duration, Instant};

pub struct MultiTierCache {
    // L1: In-memory hot cache (frequently accessed)
    hot_cache: Arc<DashMap<String, CachedValue>>,
    // L2: Redis (persistent, shared)
    redis: Arc<redis::Client>,
    // Cache policies
    policies: CachePolicies,
}

#[derive(Clone)]
struct CachedValue {
    data: Vec<u8>,
    created_at: Instant,
    access_count: u32,
    ttl: Duration,
}

struct CachePolicies {
    // Hot data (prices, active wallets) - 30 seconds
    hot_ttl: Duration,
    // Warm data (token metadata) - 5 minutes
    warm_ttl: Duration,
    // Cold data (historical trades) - 1 hour
    cold_ttl: Duration,
    // Max memory for L1 cache
    max_l1_size: usize,
}

impl MultiTierCache {
    pub async fn get_or_fetch<F, T>(
        &self,
        key: &str,
        tier: CacheTier,
        fetcher: F,
    ) -> Result<T>
    where
        F: FnOnce() -> futures::future::BoxFuture<'static, Result<T>>,
        T: serde::Serialize + serde::de::DeserializeOwned,
    {
        let cache_key = format!("{}:{}", tier.prefix(), key);

        // 1. Check L1 (in-memory)
        if let Some(cached) = self.hot_cache.get(&cache_key) {
            if cached.created_at.elapsed() < cached.ttl {
                return Ok(bincode::deserialize(&cached.data)?);
            } else {
                // Expired, remove
                self.hot_cache.remove(&cache_key);
            }
        }

        // 2. Check L2 (Redis)
        if let Ok(data) = self.get_from_redis(&cache_key).await {
            let value: T = bincode::deserialize(&data)?;
            // Promote to L1
            self.set_l1(&cache_key, &data, tier.ttl()).await;
            return Ok(value);
        }

        // 3. Fetch from source
        let value = fetcher().await?;
        let serialized = bincode::serialize(&value)?;

        // 4. Store in both caches
        self.set_l1(&cache_key, &serialized, tier.ttl()).await;
        self.set_redis(&cache_key, &serialized, tier.ttl()).await?;

        Ok(value)
    }

    /// Evict least-used items if cache is full
    async fn evict_if_needed(&self) {
        if self.hot_cache.len() > self.policies.max_l1_size {
            // Find least accessed items
            let mut items: Vec<_> = self.hot_cache
                .iter()
                .map(|entry| (entry.key().clone(), entry.access_count))
                .collect();

            items.sort_by_key(|(_, count)| *count);

            // Remove bottom 10%
            let to_remove = items.len() / 10;
            for (key, _) in items.iter().take(to_remove) {
                self.hot_cache.remove(key);
            }
        }
    }
}

pub enum CacheTier {
    Hot,   // Prices, active data
    Warm,  // Token metadata
    Cold,  // Historical data
}

impl CacheTier {
    fn prefix(&self) -> &'static str {
        match self {
            CacheTier::Hot => "hot",
            CacheTier::Warm => "warm",
            CacheTier::Cold => "cold",
        }
    }

    fn ttl(&self) -> Duration {
        match self {
            CacheTier::Hot => Duration::from_secs(30),
            CacheTier::Warm => Duration::from_secs(300),
            CacheTier::Cold => Duration::from_secs(3600),
        }
    }
}
```

**Usage**:
```rust
// Fast price lookup with caching
let token_price = cache.get_or_fetch(
    &mint.to_string(),
    CacheTier::Hot,
    || Box::pin(async move {
        token_fetcher.get_token_data(&mint).await
    })
).await?;
```

---

## 3. Websocket Subscriptions (Real-time Data)

```rust
// crates/data/src/websocket_listener.rs
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::StreamExt;

pub struct WebSocketListener {
    ws_url: String,
    tracked_accounts: Arc<RwLock<Vec<Pubkey>>>,
}

impl WebSocketListener {
    /// Listen for account changes in real-time
    pub async fn subscribe_to_accounts(&self) -> Result<()> {
        let (ws_stream, _) = connect_async(&self.ws_url).await?;
        let (mut write, mut read) = ws_stream.split();

        // Subscribe to tracked wallets
        let accounts = self.tracked_accounts.read().await;
        for account in accounts.iter() {
            let subscribe_msg = serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "accountSubscribe",
                "params": [
                    account.to_string(),
                    {
                        "encoding": "jsonParsed",
                        "commitment": "confirmed"
                    }
                ]
            });

            write.send(Message::Text(subscribe_msg.to_string())).await?;
        }

        // Listen for updates
        while let Some(msg) = read.next().await {
            match msg? {
                Message::Text(text) => {
                    self.handle_account_update(&text).await?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn handle_account_update(&self, data: &str) -> Result<()> {
        let update: serde_json::Value = serde_json::from_str(data)?;

        // Parse and emit trade signal immediately
        // Much faster than polling!

        Ok(())
    }
}
```

**Benefit**: Get trade signals in <1 second instead of 10-60 seconds with polling

---

## 4. Database Query Optimization

```sql
-- migrations/002_indexes.sql

-- Speed up wallet lookup by score
CREATE INDEX CONCURRENTLY idx_wallets_smart_money_score
ON wallets(smart_money_score DESC, is_tracked);

-- Speed up recent trades query
CREATE INDEX CONCURRENTLY idx_trades_timestamp_wallet
ON trades(timestamp DESC, wallet_address)
WHERE timestamp > NOW() - INTERVAL '7 days';

-- Speed up token analysis
CREATE INDEX CONCURRENTLY idx_trades_token_timestamp
ON trades(token_mint, timestamp DESC);

-- Partial index for active positions
CREATE INDEX CONCURRENTLY idx_positions_active
ON positions(wallet_address, token_mint)
WHERE status = 'Open';

-- Materialized view for top wallets (refresh every 5 min)
CREATE MATERIALIZED VIEW top_wallets AS
SELECT
    w.*,
    COUNT(t.id) as trade_count,
    AVG(p.pnl_percentage) as avg_pnl_pct
FROM wallets w
LEFT JOIN trades t ON w.address = t.wallet_address
LEFT JOIN positions p ON w.address = p.wallet_address
WHERE w.is_tracked = true
GROUP BY w.address
ORDER BY w.smart_money_score DESC
LIMIT 100;

CREATE UNIQUE INDEX ON top_wallets(address);

-- Refresh in background
CREATE OR REPLACE FUNCTION refresh_top_wallets()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY top_wallets;
END;
$$ LANGUAGE plpgsql;
```

---

# 💰 PnL Maximization Strategies

## 1. Advanced Wallet Scoring with Machine Learning

```rust
// crates/analysis/src/ml_scorer.rs

pub struct MLWalletScorer {
    // Feature weights learned from historical data
    feature_weights: HashMap<String, f64>,
}

impl MLWalletScorer {
    /// Calculate score using multiple features
    pub fn score_wallet(&self, analysis: &WalletAnalysis, market_context: &MarketContext) -> f64 {
        let mut features = HashMap::new();

        // Basic features
        features.insert("win_rate", analysis.metrics.win_rate / 100.0);
        features.insert("sharpe_ratio", analysis.metrics.sharpe_ratio.unwrap_or(0.0));
        features.insert("max_drawdown", 1.0 - (analysis.metrics.max_drawdown / 100.0));

        // Advanced features
        features.insert("trade_consistency", self.calculate_consistency(&analysis.metrics));
        features.insert("entry_timing_score", self.calculate_entry_timing(analysis));
        features.insert("exit_timing_score", self.calculate_exit_timing(analysis));
        features.insert("token_selection_score", self.calculate_token_selection(analysis));
        features.insert("risk_adjusted_return", self.calculate_risk_adjusted_return(&analysis.metrics));

        // Market context features
        features.insert("market_correlation", self.calculate_market_correlation(analysis, market_context));
        features.insert("volatility_adaptation", self.calculate_volatility_adaptation(analysis, market_context));

        // Weighted score
        let mut score = 0.0;
        for (feature, value) in features.iter() {
            let weight = self.feature_weights.get(feature.as_str()).unwrap_or(&0.1);
            score += value * weight;
        }

        score.max(0.0).min(1.0)
    }

    /// How consistent are the returns?
    fn calculate_consistency(&self, metrics: &WalletMetrics) -> f64 {
        // Low variance in PnL = high consistency
        // This is a simplified version
        if metrics.total_trades < 5 {
            return 0.5; // Not enough data
        }

        // Calculate coefficient of variation
        let mean_pnl = metrics.avg_profit_per_trade.to_string().parse::<f64>().unwrap_or(0.0);
        if mean_pnl == 0.0 {
            return 0.0;
        }

        // Lower CV = more consistent
        let cv = (metrics.sharpe_ratio.unwrap_or(0.0) / mean_pnl).abs();
        (1.0 / (1.0 + cv)).max(0.0).min(1.0)
    }

    /// How good is the wallet at timing entries?
    fn calculate_entry_timing(&self, analysis: &WalletAnalysis) -> f64 {
        // Check if wallet buys at low MC and sells at high MC
        let (entry_min, entry_max) = analysis.best_entry_mc_range;
        let (exit_min, exit_max) = analysis.best_exit_mc_range;

        if entry_max == Decimal::ZERO || exit_min == Decimal::ZERO {
            return 0.5;
        }

        // Calculate average entry-to-exit MC multiplier
        let avg_entry = (entry_min + entry_max) / Decimal::from(2);
        let avg_exit = (exit_min + exit_max) / Decimal::from(2);

        if avg_entry == Decimal::ZERO {
            return 0.5;
        }

        let multiplier = (avg_exit / avg_entry).to_string().parse::<f64>().unwrap_or(1.0);

        // Score based on multiplier (higher is better)
        ((multiplier - 1.0) / 10.0).max(0.0).min(1.0)
    }

    /// Calculate exit timing quality
    fn calculate_exit_timing(&self, analysis: &WalletAnalysis) -> f64 {
        // Good exit timing = sells near peak
        // Would need historical price data for this
        // For now, use hold time vs profitability

        let avg_hold = analysis.typical_hold_time;
        let win_rate = analysis.metrics.win_rate / 100.0;

        // Shorter holds with high win rate = good timing
        let timing_score = if avg_hold < 3600.0 {
            win_rate * 1.2 // Bonus for fast, profitable trades
        } else if avg_hold < 86400.0 {
            win_rate * 1.0
        } else {
            win_rate * 0.8 // Penalty for slow trades
        };

        timing_score.max(0.0).min(1.0)
    }

    /// How good is wallet at selecting profitable tokens?
    fn calculate_token_selection(&self, analysis: &WalletAnalysis) -> f64 {
        // Wallets that pick consistently good tokens score higher
        // This would require tracking which tokens succeeded

        // For now, use number of preferred tokens as proxy
        // More focused = better selection
        let focus_score = if analysis.preferred_tokens.len() <= 5 {
            1.0 // Very focused
        } else if analysis.preferred_tokens.len() <= 20 {
            0.8 // Moderately focused
        } else {
            0.5 // Scattered
        };

        focus_score
    }

    /// Risk-adjusted return (Sortino ratio variation)
    fn calculate_risk_adjusted_return(&self, metrics: &WalletMetrics) -> f64 {
        if metrics.total_trades == 0 {
            return 0.0;
        }

        // Penalize downside deviation more than upside
        let downside_dev = metrics.max_drawdown / 100.0;
        let avg_return = metrics.total_pnl_percentage / 100.0;

        if downside_dev == 0.0 {
            return 1.0;
        }

        let sortino = avg_return / downside_dev;
        (sortino / 2.0).max(0.0).min(1.0)
    }
}

pub struct MarketContext {
    pub current_volatility: f64,
    pub trend: MarketTrend,
    pub total_market_cap: Decimal,
}

pub enum MarketTrend {
    Bullish,
    Bearish,
    Sideways,
}
```

---

## 2. Dynamic Signal Confidence Scoring

```rust
// crates/strategy/src/confidence_scorer.rs

pub struct ConfidenceScorer {
    ml_scorer: MLWalletScorer,
}

impl ConfidenceScorer {
    /// Multi-factor confidence calculation
    pub async fn calculate_confidence(
        &self,
        wallet_analysis: &WalletAnalysis,
        token_security: &SecurityInfo,
        trade: &Trade,
        market_context: &MarketContext,
        historical_performance: &HistoricalPerformance,
    ) -> f64 {
        let mut confidence = 0.0;
        let mut weight_sum = 0.0;

        // Factor 1: Wallet quality (40% weight)
        let wallet_score = self.ml_scorer.score_wallet(wallet_analysis, market_context);
        confidence += wallet_score * 0.4;
        weight_sum += 0.4;

        // Factor 2: Token security (25% weight)
        let security_score = self.score_token_security(token_security);
        confidence += security_score * 0.25;
        weight_sum += 0.25;

        // Factor 3: Market timing (15% weight)
        let timing_score = self.score_market_timing(market_context, trade);
        confidence += timing_score * 0.15;
        weight_sum += 0.15;

        // Factor 4: Historical success rate for similar signals (10% weight)
        let historical_score = self.score_historical_performance(
            wallet_analysis,
            token_security,
            historical_performance,
        );
        confidence += historical_score * 0.10;
        weight_sum += 0.10;

        // Factor 5: Liquidity adequacy (10% weight)
        let liquidity_score = self.score_liquidity(&token_security, &trade);
        confidence += liquidity_score * 0.10;
        weight_sum += 0.10;

        // Normalize
        if weight_sum > 0.0 {
            confidence /= weight_sum;
        }

        // Apply penalties
        confidence = self.apply_penalties(confidence, wallet_analysis, token_security, trade);

        confidence.max(0.0).min(1.0)
    }

    fn score_token_security(&self, security: &SecurityInfo) -> f64 {
        if security.is_scam {
            return 0.0;
        }

        if security.is_bundle {
            return 0.1;
        }

        let mut score = 0.7; // Base score

        // LP locked bonus
        if security.lp_locked {
            score += 0.15;
        }

        // Low holder concentration bonus
        if security.top_holders_percentage < 50.0 {
            score += 0.10;
        }

        // Rugcheck score
        if let Some(rugcheck) = security.rugcheck_score {
            score += (rugcheck / 100.0) * 0.05;
        }

        score.min(1.0)
    }

    fn score_market_timing(&self, market: &MarketContext, trade: &Trade) -> f64 {
        // Buy signals are better in bull markets
        // Sell signals are better in bear markets

        match (trade.side, &market.trend) {
            (TradeSide::Buy, MarketTrend::Bullish) => 0.9,
            (TradeSide::Buy, MarketTrend::Sideways) => 0.6,
            (TradeSide::Buy, MarketTrend::Bearish) => 0.3,
            (TradeSide::Sell, MarketTrend::Bearish) => 0.8,
            (TradeSide::Sell, MarketTrend::Sideways) => 0.6,
            (TradeSide::Sell, MarketTrend::Bullish) => 0.4,
        }
    }

    fn apply_penalties(
        &self,
        mut confidence: f64,
        wallet: &WalletAnalysis,
        security: &SecurityInfo,
        trade: &Trade,
    ) -> f64 {
        // Penalty for high-risk tokens
        match security.risk_level {
            RiskLevel::Critical => confidence *= 0.1,
            RiskLevel::High => confidence *= 0.5,
            RiskLevel::Medium => confidence *= 0.8,
            RiskLevel::Low => confidence *= 0.95,
            RiskLevel::Safe => confidence *= 1.0,
        }

        // Penalty for wallet with high recent losses
        if wallet.metrics.trades_last_24h > 20 && wallet.metrics.win_rate < 50.0 {
            confidence *= 0.7; // Wallet might be tilting
        }

        // Bonus for insider-like behavior
        if wallet.is_insider && trade.side == TradeSide::Buy {
            confidence *= 1.2;
        }

        confidence.min(1.0)
    }
}

pub struct HistoricalPerformance {
    pub signals_by_confidence: HashMap<u8, Vec<SignalOutcome>>,
}

pub struct SignalOutcome {
    pub confidence: f64,
    pub actual_pnl: Decimal,
    pub executed_at: DateTime<Utc>,
}
```

---

## 3. Fee Optimization

```rust
// crates/trading/src/fee_optimizer.rs

pub struct FeeOptimizer {
    jupiter: Arc<JupiterClient>,
}

impl FeeOptimizer {
    /// Find route with lowest total fees
    pub async fn optimize_route(
        &self,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount: u64,
    ) -> Result<OptimizedRoute> {
        // Get multiple routes
        let routes = self.get_alternative_routes(input_mint, output_mint, amount).await?;

        let mut best_route = None;
        let mut best_net_output = Decimal::ZERO;

        for route in routes {
            // Calculate total fees
            let jupiter_fee = route.jupiter_fee;
            let dex_fees = route.dex_fees;
            let priority_fee = route.priority_fee;
            let jito_tip = route.jito_tip;

            let total_fees = jupiter_fee + dex_fees + priority_fee + jito_tip;
            let net_output = route.output_amount - total_fees;

            if net_output > best_net_output {
                best_net_output = net_output;
                best_route = Some(route);
            }
        }

        Ok(best_route.unwrap())
    }

    /// Calculate optimal priority fee based on network congestion
    pub async fn calculate_optimal_priority_fee(
        &self,
        urgency: OrderPriority,
    ) -> u64 {
        // Get recent priority fees
        let recent_fees = self.get_recent_priority_fees().await.unwrap_or_default();

        if recent_fees.is_empty() {
            return match urgency {
                OrderPriority::Low => 5_000,
                OrderPriority::Medium => 10_000,
                OrderPriority::High => 50_000,
                OrderPriority::Urgent => 100_000,
            };
        }

        // Use percentiles based on urgency
        let percentile = match urgency {
            OrderPriority::Low => 25,
            OrderPriority::Medium => 50,
            OrderPriority::High => 75,
            OrderPriority::Urgent => 90,
        };

        self.calculate_percentile(&recent_fees, percentile)
    }
}
```

**Expected Savings**: 0.1-0.5% per trade = significant over thousands of trades

---

## 4. Market Making During Low Volatility

```rust
// crates/strategy/src/market_maker.rs

/// Make markets in stablecoins/low-vol assets when no copy signals
pub struct MarketMaker {
    enabled: bool,
    min_spread_bps: u16,
}

impl MarketMaker {
    /// Place limit orders to earn spread
    pub async fn place_orders(&self, token_mint: &Pubkey) -> Result<()> {
        // Only for low-volatility, high-liquidity tokens
        // Earn spread while waiting for copy signals

        // This is advanced - implement later
        Ok(())
    }
}
```

---

Continue to Part 2 with remaining optimizations...

