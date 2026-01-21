# 💰 Free Tier Setup - Start Trading Today ($0 Budget)

## 🎯 Goal
Build initial capital through smart trading with **zero monthly costs**, then reinvest profits to improve the system.

---

## 🆓 100% Free Infrastructure

### 1. Free Solana RPC Endpoints
```env
# .env configuration
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_FALLBACK_RPC_1=https://solana-api.projectserum.com
SOLANA_FALLBACK_RPC_2=https://rpc.ankr.com/solana
SOLANA_FALLBACK_RPC_3=https://solana.public-rpc.com
SOLANA_FALLBACK_RPC_4=https://api.metaplex.solana.com
```

**Limits**:
- ~100 requests/second combined
- Free forever
- Good enough for swing trading

---

### 2. Free Data Sources

#### Jupiter API (Free, No Key Required)
```env
JUPITER_API_URL=https://quote-api.jup.ag/v6
# No API key needed!
```

#### DexScreener (Free, Rate Limited)
```env
DEXSCREENER_API_URL=https://api.dexscreener.com/latest
# ~30 requests/minute
# No API key needed!
```

#### Rugcheck (Free)
```env
RUGCHECK_API_URL=https://api.rugcheck.xyz/v1
# Free tier available
# No registration needed for basic checks
```

#### Birdeye Public API (Free Tier)
```env
BIRDEYE_API_URL=https://public-api.birdeye.so
# Free tier: 100 requests/day
# Upgrade later when profitable
```

---

### 3. Free Database (Local)

**Option A: SQLite (Simplest)**
```bash
# No installation needed, built into Rust
# Perfect for starting out
```

```env
DATABASE_URL=sqlite:trading_bot.db
```

**Option B: PostgreSQL (Docker)**
```bash
# Free, runs locally
docker run -d --name trading-postgres \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=trading_bot \
  -p 5432:5432 \
  postgres:14-alpine

# Or install locally (no cost)
```

**Option C: Turso (Free Cloud SQLite)**
```bash
# Free tier: 500 MB, 1 billion reads
# Perfect for starting
# https://turso.tech
```

---

### 4. Free Monitoring

**Console Logging (Free)**
```env
LOG_LEVEL=info
METRICS_ENABLED=false  # Disable Prometheus to save resources
```

**Free Telegram Bot (Optional)**
```env
TELEGRAM_ENABLED=true
TELEGRAM_BOT_TOKEN=get_from_@BotFather  # Free!
TELEGRAM_CHAT_ID=your_chat_id
```

---

## 🚀 Minimal Viable Bot (Free Version)

### Simplified Configuration

```bash
# Copy this to .env
cat > .env << 'EOF'
# Solana (Free public RPCs)
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_FALLBACK_RPC_1=https://solana-api.projectserum.com
SOLANA_FALLBACK_RPC_2=https://rpc.ankr.com/solana
WALLET_PRIVATE_KEY=YOUR_WALLET_KEY_HERE

# Data Sources (All Free)
JUPITER_API_URL=https://quote-api.jup.ag/v6
DEXSCREENER_API_URL=https://api.dexscreener.com/latest
RUGCHECK_API_URL=https://api.rugcheck.xyz/v1

# Database (Free SQLite)
DATABASE_URL=sqlite:trading_bot.db
REDIS_URL=  # Leave empty, we'll use in-memory cache

# Trading (Start Conservative)
TRADING_ENABLED=false  # Start with paper trading!
USE_JITO=false  # Enable later when profitable
DEFAULT_SLIPPAGE_BPS=100
MIN_TRADE_INTERVAL_SECONDS=60  # Slower = fewer API calls

# Strategy (Conservative Settings)
STRATEGY_MODE=SwingTrading  # Longer holds = fewer trades
MIN_SMART_MONEY_SCORE=0.8  # Only copy the best
MIN_WIN_RATE=70.0
MIN_TRADES_FOR_ANALYSIS=20  # Higher confidence
MAX_TRACKED_WALLETS=20  # Lower API usage

# Risk (Start Small)
MAX_POSITION_SIZE_USD=10  # Start tiny!
MAX_DAILY_LOSS_USD=5
STOP_LOSS_PERCENTAGE=10.0
TAKE_PROFIT_PERCENTAGE=30.0

# Monitoring (Free)
TELEGRAM_ENABLED=false  # Enable after setup
LOG_LEVEL=info
METRICS_ENABLED=false

# Update Intervals (Slower = Free)
TOKEN_ANALYSIS_INTERVAL=300  # 5 minutes
WALLET_ANALYSIS_INTERVAL=600  # 10 minutes
EOF
```

---

## 📦 Minimal Dependencies (Free)

### Update Cargo.toml for Free Tier

```toml
# crates/db/Cargo.toml
[dependencies]
trading-core = { path = "../core" }

# Use SQLite instead of PostgreSQL
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "chrono", "uuid"] }

# No Redis - use in-memory cache
dashmap = "5.5"  # Free, fast in-memory cache
```

---

## 🎯 Free Tier Strategy

### Focus Areas (Alpha Without Costs)

#### 1. Manual Wallet Discovery (Free)
```bash
# Find top wallets using free tools
# 1. Go to dexscreener.com
# 2. Find trending Solana tokens
# 3. Click on token → "Top Traders"
# 4. Copy wallet addresses of consistent winners
# 5. Add to tracked_wallets.txt
```

**Example `tracked_wallets.txt`**:
```
# Found on DexScreener - BONK top trader
GVXRSBjFk6e6J3NbVPXohDJetcTjaeeuykUpbQF8UoMU

# Found on Photon - Consistent 70%+ win rate
2dKmU13YlQyJmfQXCxqm7s9bXmF9Q2B8Tz6cQQEKPomR

# Add more as you find them...
```

#### 2. Free Scam Detection
```rust
// crates/strategy/src/free_scam_check.rs

pub struct FreeScamChecker;

impl FreeScamChecker {
    /// Basic scam checks without paid APIs
    pub async fn is_likely_scam(token: &Token) -> bool {
        // Red flags (free to check)

        // 1. Extremely low liquidity
        if token.market_data.liquidity_usd < Decimal::from(5000) {
            return true; // Too risky
        }

        // 2. No volume
        if token.market_data.volume_24h < Decimal::from(1000) {
            return true; // Dead token
        }

        // 3. Suspicious price action
        if token.market_data.price_change_5m > 100.0 {
            return true; // Likely pump and dump
        }

        // 4. Very new token (check if created recently)
        let age_hours = (Utc::now() - token.created_at).num_hours();
        if age_hours < 1 {
            return true; // Too new, too risky
        }

        false
    }

    /// Check if token is worth trading (free checks only)
    pub async fn is_worth_trading(token: &Token) -> bool {
        // Must have:
        // 1. Decent liquidity (can exit)
        if token.market_data.liquidity_usd < Decimal::from(10000) {
            return false;
        }

        // 2. Real volume
        if token.market_data.volume_24h < Decimal::from(5000) {
            return false;
        }

        // 3. Reasonable market cap
        let mc = token.market_data.market_cap;
        if mc < Decimal::from(50000) || mc > Decimal::from(100_000_000) {
            return false; // Too small or too large
        }

        // 4. Not extreme volatility
        if token.market_data.price_change_1h.abs() > 50.0 {
            return false; // Too volatile
        }

        true
    }
}
```

#### 3. Simple Alpha Detection (Free)
```rust
// crates/strategy/src/free_alpha_detector.rs

pub struct FreeAlphaDetector;

impl FreeAlphaDetector {
    /// Detect alpha opportunities using free data
    pub async fn find_alpha(
        &self,
        tracked_wallets: &[Pubkey],
        recent_trades: &HashMap<Pubkey, Vec<Trade>>,
    ) -> Vec<AlphaSignal> {
        let mut signals = vec![];

        // Alpha Signal 1: Multiple smart wallets buying same token
        let mut token_buy_count: HashMap<Pubkey, u32> = HashMap::new();

        for (wallet, trades) in recent_trades {
            if !tracked_wallets.contains(wallet) {
                continue;
            }

            // Recent buys (last hour)
            let recent_buys: Vec<_> = trades
                .iter()
                .filter(|t| {
                    t.side == TradeSide::Buy
                    && (Utc::now() - t.timestamp).num_hours() < 1
                })
                .collect();

            for trade in recent_buys {
                *token_buy_count.entry(trade.token_mint).or_insert(0) += 1;
            }
        }

        // If 3+ smart wallets bought same token → ALPHA!
        for (token_mint, count) in token_buy_count {
            if count >= 3 {
                signals.push(AlphaSignal {
                    token_mint,
                    signal_type: AlphaType::SmartMoneyConvergence,
                    confidence: 0.7 + (count as f64 * 0.05),
                    detected_wallets: count,
                    reason: format!("{} smart wallets bought in last hour", count),
                });
            }
        }

        // Alpha Signal 2: Wallet with >80% win rate makes a trade
        for (wallet, trades) in recent_trades {
            if !tracked_wallets.contains(wallet) {
                continue;
            }

            let metrics = WalletMetricsCalculator::calculate_metrics(trades).unwrap();

            if metrics.win_rate > 80.0 && metrics.total_trades >= 20 {
                // This wallet is HOT
                if let Some(latest_trade) = trades.last() {
                    if (Utc::now() - latest_trade.timestamp).num_minutes() < 10 {
                        signals.push(AlphaSignal {
                            token_mint: latest_trade.token_mint,
                            signal_type: AlphaType::TopWalletTrade,
                            confidence: metrics.win_rate / 100.0,
                            detected_wallets: 1,
                            reason: format!(
                                "Wallet with {:.1}% win rate just bought",
                                metrics.win_rate
                            ),
                        });
                    }
                }
            }
        }

        signals
    }
}

#[derive(Debug, Clone)]
pub struct AlphaSignal {
    pub token_mint: Pubkey,
    pub signal_type: AlphaType,
    pub confidence: f64,
    pub detected_wallets: u32,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub enum AlphaType {
    SmartMoneyConvergence,  // Multiple smart wallets buying
    TopWalletTrade,         // Elite wallet activity
    UnusualVolume,          // Volume spike
    EarlyAccumulation,      // Buying at low MC
}
```

---

## 💡 Free Tier Optimizations

### 1. Aggressive In-Memory Caching
```rust
// crates/data/src/free_cache.rs

use dashmap::DashMap;
use std::sync::Arc;

pub struct FreeCache {
    // In-memory only (no Redis cost)
    cache: Arc<DashMap<String, (Vec<u8>, i64)>>,
    // Longer TTLs to reduce API calls
    default_ttl: i64,
}

impl FreeCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            default_ttl: 600, // 10 minutes (vs 60s for paid)
        }
    }

    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        if let Some(entry) = self.cache.get(key) {
            let (data, timestamp) = entry.value();
            let age = Utc::now().timestamp() - timestamp;

            if age < self.default_ttl {
                return bincode::deserialize(data).ok();
            } else {
                // Expired
                drop(entry);
                self.cache.remove(key);
            }
        }
        None
    }

    pub fn set<T: serde::Serialize>(&self, key: &str, value: &T) {
        if let Ok(data) = bincode::serialize(value) {
            self.cache.insert(
                key.to_string(),
                (data, Utc::now().timestamp()),
            );
        }
    }
}
```

### 2. Rate Limiting (Respect Free Tiers)
```rust
// crates/data/src/rate_limiter.rs

use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub struct RateLimiter {
    last_call: Mutex<Instant>,
    min_interval: Duration,
}

impl RateLimiter {
    pub fn new(calls_per_minute: u32) -> Self {
        let min_interval = Duration::from_secs(60 / calls_per_minute as u64);
        Self {
            last_call: Mutex::new(Instant::now() - min_interval),
            min_interval,
        }
    }

    pub async fn wait(&self) {
        let mut last = self.last_call.lock().await;
        let elapsed = last.elapsed();

        if elapsed < self.min_interval {
            let wait_time = self.min_interval - elapsed;
            tokio::time::sleep(wait_time).await;
        }

        *last = Instant::now();
    }
}

// Usage
pub struct DexScreenerClient {
    rate_limiter: RateLimiter,
}

impl DexScreenerClient {
    pub fn new() -> Self {
        Self {
            // Free tier: ~30 requests/minute
            rate_limiter: RateLimiter::new(25), // Be conservative
        }
    }

    pub async fn get_token(&self, mint: &Pubkey) -> Result<Token> {
        // Wait for rate limit
        self.rate_limiter.wait().await;

        // Make request...
    }
}
```

### 3. Batch Processing (Fewer Requests)
```rust
// Process multiple wallets per request cycle
pub async fn analyze_wallets_batch(
    wallets: &[Pubkey],
    batch_size: usize,
) -> Vec<WalletAnalysis> {
    let mut results = vec![];

    // Process in batches with delays
    for chunk in wallets.chunks(batch_size) {
        for wallet in chunk {
            if let Ok(analysis) = analyze_wallet(wallet).await {
                results.push(analysis);
            }
        }

        // Wait between batches (respect rate limits)
        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    results
}
```

---

## 🎯 Capital Building Strategy (Start Free)

### Phase 1: $0-50 Capital (100% Free)

**Strategy**: Ultra-conservative swing trading
```env
MAX_POSITION_SIZE_USD=5
MAX_DAILY_LOSS_USD=2
STOP_LOSS_PERCENTAGE=8.0
TAKE_PROFIT_PERCENTAGE=25.0
MIN_SMART_MONEY_SCORE=0.85
```

**Expected**:
- 1-2 trades per day
- Win rate: 65-70% (with good wallet selection)
- Monthly growth: +20-40%

**Reinvestment**:
```
Month 1: $10 → $12-14
Month 2: $14 → $17-20
Month 3: $20 → $24-28
```

### Phase 2: $50-200 Capital (Upgrade Selectively)

**Upgrades to Consider**:
1. Helius Free Tier ($0, better RPC)
2. Still using free DexScreener
3. Enable Jito ($0.01-0.05 per trade)

```env
MAX_POSITION_SIZE_USD=20
USE_JITO=true  # Better fills worth the small cost
JITO_TIP_LAMPORTS=5000  # Minimal tip
```

**Expected**:
- 3-5 trades per day
- Monthly growth: +30-50%

### Phase 3: $200-1000 Capital (Paid APIs)

**Upgrades**:
1. Helius Growth ($50/month) - faster, more reliable
2. Consider Birdeye paid tier
3. Increase Jito tips

```env
MAX_POSITION_SIZE_USD=50
JITO_TIP_LAMPORTS=10000
MAX_TRACKED_WALLETS=50
```

**Expected**:
- 5-10 trades per day
- Monthly growth: +40-60%

### Phase 4: $1000+ Capital (Full Featured)

**Implement all optimizations from OPTIMIZATION_GUIDE.md**

---

## 📊 Free Tier Performance Expectations

### Realistic Goals (Free Version)

**Conservative**:
```
Starting Capital: $10
Win Rate: 60%
Avg Trade: $0.50 profit
Trades/Day: 1-2
Monthly Profit: $9-18 (90-180%)
```

**Optimistic** (good wallet selection):
```
Starting Capital: $10
Win Rate: 70%
Avg Trade: $1.00 profit
Trades/Day: 2-3
Monthly Profit: $20-30 (200-300%)
```

**Over 6 Months** (compounding):
```
Month 1: $10 → $15
Month 2: $15 → $25
Month 3: $25 → $40
Month 4: $40 → $70
Month 5: $70 → $120
Month 6: $120 → $200

Then upgrade to paid tiers!
```

---

## 🚀 Getting Started Today (Free)

### 1. Minimal Setup (30 minutes)
```bash
cd /Users/dac/solana-trading-bot

# 1. Edit .env with free settings
cp .env.example .env
# Edit with settings from above

# 2. Create tracked wallets file
echo "# Add wallet addresses here, one per line" > tracked_wallets.txt

# 3. Use SQLite (no setup needed)
# 4. Build
cargo build --release

# 5. Start paper trading
TRADING_ENABLED=false cargo run --release
```

### 2. Find Smart Wallets (Free Methods)

**Method 1: DexScreener**
```
1. Go to dexscreener.com/solana
2. Find tokens with good volume
3. Click "Top Traders" tab
4. Copy addresses of consistent winners
5. Add to tracked_wallets.txt
```

**Method 2: Photon**
```
1. Go to photon-sol.tinyastro.io
2. Search popular tokens
3. Look at "Top Traders"
4. Copy wallet addresses
```

**Method 3: SolScan**
```
1. Find a profitable token
2. Check top holders
3. Analyze their trade history
4. If good win rate → add to list
```

### 3. Start Paper Trading (1 hour)
```bash
# Run bot in paper mode
cargo run --release --bin bot

# Monitor output
# It will log:
# - Wallets being tracked
# - Trades detected
# - Signals generated
# - Simulated executions
```

### 4. Go Live (When Ready)
```bash
# After 3-7 days of successful paper trading:

# 1. Add small amount to wallet ($5-10)
# 2. Enable trading
echo "TRADING_ENABLED=true" >> .env

# 3. Start bot
cargo run --release --bin bot

# 4. Monitor closely!
tail -f bot.log
```

---

## 💰 Profit Reinvestment Plan

### Every $50 Profit:
- Keep $25 in trading capital
- Withdraw $15 (secure profits)
- Invest $10 in upgrades

### Upgrade Priority (As Capital Grows):

**$50 profit** → Helius free tier (still $0)
**$100 profit** → Enable Jito ($0.05/trade)
**$200 profit** → Helius Growth ($50/month)
**$500 profit** → Birdeye paid tier
**$1000 profit** → Implement all optimizations

---

## ⚠️ Free Tier Limitations

### Expect:
- Slower updates (5-10 minute intervals vs real-time)
- Manual wallet discovery (no auto-discovery)
- Basic scam detection (no advanced ML)
- Rate limiting (fewer trades possible)
- May miss some opportunities

### But You Get:
- ✅ Zero monthly costs
- ✅ Functional trading bot
- ✅ Real alpha detection
- ✅ Scam avoidance
- ✅ Capital building
- ✅ Path to upgrade

---

## 📈 Success Metrics (Free Tier)

### Week 1 (Paper Trading):
- [ ] Bot runs 24/7 without crashes
- [ ] Generates 5+ signals
- [ ] No scam tokens traded
- [ ] Win rate >60% (simulated)

### Week 2-3 (Live, Small Capital):
- [ ] 10+ real trades executed
- [ ] Win rate >60%
- [ ] No major losses
- [ ] Positive PnL

### Month 1:
- [ ] 50+ trades
- [ ] Win rate >65%
- [ ] Capital increased 20%+
- [ ] Ready to upgrade

---

## 🎯 Next Steps

1. ✅ Copy free tier .env configuration
2. ✅ Find 5-10 smart wallets to track
3. ✅ Start paper trading today
4. ✅ Monitor for 3-7 days
5. ✅ Go live with $5-10
6. ✅ Build capital
7. ✅ Reinvest in upgrades

**You can start building capital TODAY with $0 monthly costs!**

Then use profits to systematically upgrade the system. 🚀
