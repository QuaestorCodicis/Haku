# Solana Smart Money Trading Bot

An advanced, fully automated Solana trading assistant that analyzes top-performing wallets, detects insider activity, and executes copy trades to maximize PnL consistently and safely.

## 🎉 **LATEST: v2.0 Enhanced Bot with Complete Position Management!**

The bot now features:
- ✅ **Multi-Wallet Convergence Detection** (85-95% confidence signals)
- ✅ **Technical Chart Analysis** (7 entry/exit patterns)
- ✅ **Automatic Position Management** (stop-loss, take-profit, trailing stops)
- ✅ **Elite Wallet Finder Tool** (discover top traders automatically)
- ✅ **Real-Time Dashboard** (live P&L tracking and statistics)

**See [WHATS_NEW.md](WHATS_NEW.md) for complete v2.0 features!**

## 🚀 Quick Links

- **[ENHANCED_BOT_QUICKSTART.md](ENHANCED_BOT_QUICKSTART.md)** - ⭐ **v2.0 Setup Guide**
- **[START_TODAY.md](START_TODAY.md)** - 💰 **Run bot in 15 minutes ($0 cost)**
- **[WHATS_NEW.md](WHATS_NEW.md)** - 🆕 **Latest Features & Changes**
- **[FREE_TIER_SETUP.md](FREE_TIER_SETUP.md)** - Complete free tier guide
- **[CAPITAL_BUILDING_PLAN.md](CAPITAL_BUILDING_PLAN.md)** - $10 to $500+ roadmap

## 📖 Complete Documentation

**Getting Started**:
- **[README.md](README.md)** - This file - features and overview
- **[QUICK_START.md](QUICK_START.md)** - 5-minute overview + learning path

**Optimizations** (Read after first profitable month):
- **[OPTIMIZATION_SUMMARY.md](OPTIMIZATION_SUMMARY.md)** - Overview of all optimizations
- **[OPTIMIZATION_GUIDE.md](OPTIMIZATION_GUIDE.md)** - Part 1: Safety, security, efficiency
- **[OPTIMIZATION_GUIDE_PART2.md](OPTIMIZATION_GUIDE_PART2.md)** - Part 2: Advanced intelligence
- **[PRIORITY_IMPLEMENTATION.md](PRIORITY_IMPLEMENTATION.md)** - 6-week implementation plan

**Development**:
- **[IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md)** - Step-by-step development guide

## 🎯 Features

### 🆕 v2.0 Enhanced Features
- **Multi-Wallet Convergence Detection**: Detect when 3+ elite wallets buy the same token
  - 85-95% confidence signals (vs 65% for single wallets)
  - Automatic confidence scoring
  - Time-window based clustering (60 minutes)

- **Technical Chart Analysis**: 7 different entry/exit patterns
  - Volume breakouts and consolidation breaks
  - Pullback buying in uptrends
  - Overbought/oversold detection
  - Early pump identification
  - Downtrend detection for exits

- **Complete Position Management**: Automated entry and exit system
  - Automatic stop-loss protection (-10% default)
  - Take-profit targets (+40% default)
  - Trailing stops for big winners
  - Time-based exits for stale positions
  - Chart-based exits on reversal signals

- **Elite Wallet Finder**: Automated discovery tool
  - Analyzes candidate wallets for performance
  - Filters for 75%+ win rate, 0.8+ smart score
  - Auto-saves top performers
  - Continuous wallet ranking

- **Real-Time Monitoring Dashboard**:
  - Live P&L tracking
  - Position monitoring with unrealized gains/losses
  - Win rate calculation
  - Big win celebrations (>$5 profit)
  - Trade statistics and analytics

### Core Capabilities
- **Smart Wallet Analysis**: Track and analyze top-performing wallets with comprehensive metrics
  - Win rate calculation
  - PnL tracking
  - Average hold time analysis
  - Entry/exit market cap optimization
  - Sharpe ratio and risk metrics

- **Scam & Bundle Detection**:
  - Integration with rugcheck.xyz API
  - Wallet clustering analysis
  - Top holder concentration checks
  - LP lock verification
  - Risk level scoring

- **Insider Activity Detection**:
  - Timing correlation analysis
  - Coordinated buying detection
  - Early accumulation patterns
  - Whale activity monitoring

- **Copy Trading Engine**:
  - Multi-factor wallet scoring
  - Confidence-based signal generation
  - Position sizing algorithms
  - Risk-adjusted trade execution

- **Trading Execution**:
  - Jupiter aggregator integration for best prices
  - MEV protection via Jito bundles
  - Automatic slippage management
  - Priority fee optimization

- **Adaptive Strategy**:
  - Dynamic mode switching (Scalping, Day Trading, Swing Trading)
  - Market condition analysis
  - Pattern recognition

- **Risk Management**:
  - Position size limits
  - Daily loss limits
  - Stop-loss and take-profit automation
  - Circuit breakers for suspicious activity
  - Liquidity requirements

## 🏗️ Architecture

```
solana-trading-bot/
├── crates/
│   ├── core/           # Core data structures and types
│   ├── data/           # Data fetching and RPC layer
│   │   ├── rpc.rs      # Solana RPC client with fallback
│   │   ├── token.rs    # DexScreener integration
│   │   ├── transaction.rs  # Transaction parsing
│   │   ├── scam_check.rs   # Rugcheck integration
│   │   └── jupiter.rs  # Jupiter swap integration
│   ├── analysis/       # Wallet and pattern analysis
│   │   ├── wallet_metrics.rs    # Performance calculation
│   │   ├── smart_money_score.rs # Smart wallet scoring
│   │   ├── insider_detection.rs # Insider activity detection
│   │   └── pattern_recognition.rs  # Trading patterns
│   ├── trading/        # Trade execution engine
│   ├── risk/           # Risk management system
│   ├── strategy/       # Decision logic
│   ├── db/             # Database layer
│   └── bot/            # Main orchestration
└── ...
```

## 📋 Prerequisites

- **Rust** (1.70+): Install from [rustup.rs](https://rustup.rs/)
- **PostgreSQL** (14+): For storing wallet data, trades, and analytics
- **Redis** (6+): For caching and real-time data
- **Solana CLI** (optional): For wallet generation and testing

## 🚀 Quick Start

### 1. Clone and Setup

```bash
git clone <your-repo>
cd solana-trading-bot

# Copy environment template
cp .env.example .env

# Edit .env with your configuration
nano .env
```

### 2. Database Setup

```bash
# Start PostgreSQL and Redis (using Docker)
docker run -d --name trading-postgres \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=trading_bot \
  -p 5432:5432 postgres:14

docker run -d --name trading-redis \
  -p 6379:6379 redis:6

# Or use your existing PostgreSQL/Redis installation
```

### 3. Generate Wallet

```bash
# Install Solana CLI if not already installed
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Generate new wallet
solana-keygen new --outfile ~/trading-bot-wallet.json

# Get the base58 private key
solana-keygen pubkey ~/trading-bot-wallet.json

# Convert to base58 private key for .env
# (Use a tool or implement in your bot to read the JSON keypair)
```

### 4. Get Free API Keys

#### Helius (Free Tier - 100k credits/month)
1. Go to [helius.dev](https://helius.dev)
2. Sign up for free account
3. Create a new project
4. Copy API key to `HELIUS_API_KEY` in `.env`
5. Use RPC: `https://rpc.helius.xyz/?api-key=YOUR_KEY`

#### Optional: Birdeye (Free Tier)
1. Go to [birdeye.so](https://birdeye.so)
2. Sign up for API access
3. Copy key to `BIRDEYE_API_KEY` in `.env`

### 5. Find Elite Wallets (New!)

```bash
# Use the automated wallet finder
cargo run --bin find-wallets

# This will analyze wallets and save elite traders to tracked_wallets.txt
```

### 6. Build and Run

```bash
# Build in release mode for performance
cargo build --release

# Run the ENHANCED bot (v2.0 - recommended)
cargo run --release --bin bot-enhanced

# Or run the original bot
cargo run --release --bin bot

# When ready for live trading, set TRADING_ENABLED=true in .env
```

## ⚙️ Configuration

### Trading Modes

**Scalping** (seconds to minutes)
```env
STRATEGY_MODE=Scalping
MIN_TRADE_INTERVAL_SECONDS=10
```

**Day Trading** (minutes to hours)
```env
STRATEGY_MODE=DayTrading
MIN_TRADE_INTERVAL_SECONDS=30
```

**Swing Trading** (hours to days)
```env
STRATEGY_MODE=SwingTrading
MIN_TRADE_INTERVAL_SECONDS=60
```

**Adaptive** (automatic switching)
```env
ADAPTIVE_MODE=true
```

### Risk Limits

Conservative:
```env
MAX_POSITION_SIZE_USD=50
MAX_DAILY_LOSS_USD=25
STOP_LOSS_PERCENTAGE=10.0
MIN_SMART_MONEY_SCORE=0.8
```

Moderate:
```env
MAX_POSITION_SIZE_USD=100
MAX_DAILY_LOSS_USD=50
STOP_LOSS_PERCENTAGE=15.0
MIN_SMART_MONEY_SCORE=0.7
```

Aggressive:
```env
MAX_POSITION_SIZE_USD=200
MAX_DAILY_LOSS_USD=100
STOP_LOSS_PERCENTAGE=20.0
MIN_SMART_MONEY_SCORE=0.6
```

## 📊 Monitoring

### Logs
```bash
# Set log level
LOG_LEVEL=debug  # trace, debug, info, warn, error

# View logs
tail -f bot.log
```

### Metrics
Access Prometheus metrics at `http://localhost:9090/metrics`

### Telegram Notifications
```env
TELEGRAM_ENABLED=true
TELEGRAM_BOT_TOKEN=your_bot_token
TELEGRAM_CHAT_ID=your_chat_id
```

## 🎯 How It Works

### 1. Wallet Discovery
- Monitor high-volume DEX transactions
- Identify wallets with consistent profits
- Track top performers in specific tokens

### 2. Analysis
- Calculate comprehensive metrics (win rate, PnL, Sharpe ratio)
- Score wallets using multi-factor algorithm
- Detect insider patterns and whale activity
- Identify best entry/exit market caps

### 3. Copy Trading Signals
- Generate signals when tracked wallets make trades
- Apply confidence scoring based on:
  - Wallet's smart money score
  - Token security check
  - Market conditions
  - Risk assessment

### 4. Execution
- Check risk limits before trading
- Get best route from Jupiter
- Execute with MEV protection (Jito)
- Monitor position and set stop-loss/take-profit

### 5. Risk Management
- Enforce position size limits
- Monitor daily P&L
- Automatically close positions at stop-loss/take-profit
- Circuit breakers for abnormal conditions

## 🔧 Development

### Build Commands
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Check code
cargo clippy

# Format code
cargo fmt
```

### Project Structure
- **core**: Data types shared across all crates
- **data**: External data fetching and parsing
- **analysis**: Wallet and pattern analysis algorithms
- **trading**: Trade execution logic
- **risk**: Risk management rules
- **strategy**: Trading strategy decision engine
- **db**: Database interactions
- **bot**: Main application orchestration

## 🛡️ Security Best Practices

1. **Start with Paper Trading**
   - Set `TRADING_ENABLED=false`
   - Monitor behavior for at least a week

2. **Use Small Amounts Initially**
   - Start with `MAX_POSITION_SIZE_USD=10`
   - Gradually increase after validation

3. **Secure Your Wallet**
   - Never commit private keys to git
   - Use environment variables
   - Consider using hardware wallet for large amounts

4. **Monitor Constantly**
   - Enable Telegram notifications
   - Check logs regularly
   - Set conservative risk limits initially

5. **Backup Your Data**
   - Regular PostgreSQL backups
   - Export important wallet analyses

## 📈 Performance Optimization

### For Low Budget (<$50/month)
- Use free RPCs with fallback
- Cache aggressively (longer TTLs)
- Limit tracked wallets to 50
- Longer analysis intervals (5+ minutes)
- Focus on higher market cap tokens

### For Better Performance
- Upgrade to paid Helius tier ($50/month)
- Reduce cache TTLs
- Track more wallets (100+)
- Shorter analysis intervals (1 minute)
- Include low cap opportunities

## 🐛 Troubleshooting

**RPC Rate Limiting**
- Add more fallback RPCs
- Increase delays between requests
- Use Helius paid tier

**Database Connection Issues**
- Check PostgreSQL is running
- Verify DATABASE_URL is correct
- Increase max_connections

**Failed Transactions**
- Increase slippage tolerance
- Increase priority fees
- Check wallet has sufficient SOL

**Missing Trades**
- Reduce MIN_TRADE_INTERVAL_SECONDS
- Lower MIN_SMART_MONEY_SCORE
- Increase MAX_TRACKED_WALLETS

## 📚 Resources

- [Solana Docs](https://docs.solana.com/)
- [Jupiter Docs](https://station.jup.ag/docs)
- [Helius RPC](https://docs.helius.dev/)
- [DexScreener API](https://docs.dexscreener.com/)
- [Rugcheck](https://rugcheck.xyz/)

## ⚠️ Disclaimer

This software is for educational and research purposes. Cryptocurrency trading carries significant risk. Only trade with funds you can afford to lose. The authors are not responsible for any financial losses.

**Key Risks:**
- Smart contracts can have bugs
- Scams and rug pulls are common
- Market volatility can cause rapid losses
- Copy trading doesn't guarantee profits
- Insider detection is probabilistic, not certain

Always do your own research (DYOR) and never invest more than you can afford to lose.

## 📝 License

MIT License - see LICENSE file for details

## 🤝 Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Submit a pull request

## 📧 Support

For issues and questions:
- Open a GitHub issue
- Check existing documentation
- Review logs for error messages

---

Built with 🦀 Rust for maximum performance and safety
