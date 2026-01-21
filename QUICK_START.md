# Quick Start Guide

Get your Solana trading bot up and running in 5 minutes!

## Prerequisites

- Rust installed (`rustup` recommended)
- Basic knowledge of Solana
- Terminal/command line access

## Step 1: Clone and Build (2 min)

```bash
# Navigate to the project
cd solana-trading-bot

# Build the project
cargo build --release
```

## Step 2: Configure (1 min)

```bash
# Create your environment file
cp .env.example .env

# Edit with your favorite editor
nano .env  # or vim, code, etc.
```

**Minimum required settings:**
```bash
# Keep these for safety!
TRADING_ENABLED=false
MAX_POSITION_SIZE_USD=10
```

Everything else has sensible defaults.

## Step 3: Add Wallets to Track (1 min)

Create `tracked_wallets.txt` and add profitable wallet addresses (one per line):

```bash
# Example
GrAkKfEpTKQuVHG2Y97Y2FF4i7y7Q5AHLK94JBy7Y5yv
4xvPGHY8Bn3xJPmCkXiYNBr8tYJYbNvE5jZ9aPXq3DzQ
```

**Where to find good wallets:**
- dexscreener.com (check top traders)
- solscan.io (analyze successful traders)
- Follow on-chain analytics

## Step 4: Run the Bot! (1 min)

```bash
cargo run --bin bot-enhanced
```

**You should see:**
```
╔═══════════════════════════════════════════════════════════════╗
║                 SOLANA TRADING BOT                            ║
║          ENHANCED FREE TIER - ACCELERATED MODE                ║
╚═══════════════════════════════════════════════════════════════╝

📝 PAPER TRADING MODE (Recommended for testing)
🔌 Initializing components...
✅ RPC connected! Slot: 123456789
📊 Tracking 2 elite wallets
🌐 Web dashboard enabled at http://localhost:3000
🚀 Starting accelerated trading loop...
```

## Step 5: Monitor (30 seconds)

Open your browser:
```
http://localhost:3000
```

You'll see:
- Real-time portfolio value
- Win rate statistics
- Recent trades
- Live updates every 5 seconds

---

## Next Steps

### Enable Telegram Notifications (Optional - 5 min)

1. Message @BotFather on Telegram
2. Create a bot: `/newbot`
3. Copy the token
4. Message @userinfobot to get your chat ID
5. Update `.env`:
   ```bash
   TELEGRAM_ENABLED=true
   TELEGRAM_BOT_TOKEN=your_token
   TELEGRAM_CHAT_ID=your_chat_id
   ```

See [TELEGRAM_SETUP.md](./TELEGRAM_SETUP.md) for details.

### Run Your First Backtest (After 10+ trades)

```bash
cargo run --bin backtest
```

See [BACKTESTING_GUIDE.md](./BACKTESTING_GUIDE.md) for details.

### Optimize Your Strategy

After collecting data, adjust these parameters in `.env`:

```bash
MIN_SMART_MONEY_SCORE=0.8  # Higher = more selective (0.7-0.9)
WALLET_ANALYSIS_INTERVAL=300  # Lower = faster (180-600)
MAX_POSITION_SIZE_USD=10  # Match your risk tolerance
```

---

## Common Issues

### "No wallets tracked"
- Create `tracked_wallets.txt`
- Add at least one valid Solana address
- One address per line

### "RPC connection failed"
- Check internet connection
- Try a different RPC in `.env`:
  ```bash
  SOLANA_RPC_URL=https://rpc.ankr.com/solana
  ```

### "Port 3000 already in use"
- Change dashboard port:
  ```bash
  DASHBOARD_PORT=3001
  ```

### Build errors
- Update Rust: `rustup update`
- Clean and rebuild: `cargo clean && cargo build`

---

## What to Expect

### First Hour
- Bot analyzes your tracked wallets
- Displays smart money scores
- Looks for trading signals
- No trades yet (building confidence)

### After Few Hours
- First signals detected
- Virtual positions opened (paper trading)
- Dashboard shows activity
- Trade history starts building

### After 1-2 Days
- 10-20 paper trades completed
- Performance metrics available
- Ready for backtesting
- Consider optimizing parameters

---

## Safety Checklist

Before enabling live trading:

- [ ] Ran in paper mode for 24+ hours
- [ ] Collected 20+ trades
- [ ] Backtested with good results
- [ ] Set reasonable position sizes
- [ ] Have stop-loss enabled
- [ ] Monitoring actively (dashboard + Telegram)
- [ ] Understand the risks

**Start with tiny positions ($1-5) when going live!**

---

## Resources

- **Full Feature List:** [V2_1_FEATURES.md](./V2_1_FEATURES.md)
- **Telegram Setup:** [TELEGRAM_SETUP.md](./TELEGRAM_SETUP.md)
- **Backtesting:** [BACKTESTING_GUIDE.md](./BACKTESTING_GUIDE.md)

---

## Need Help?

- Check the troubleshooting section above
- Review the full documentation
- Check logs for error messages
- Try in a fresh terminal session

---

**Happy Trading! 🚀**

Remember: Start with paper trading, collect data, backtest, then gradually go live with small amounts.
