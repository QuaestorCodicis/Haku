# Backtesting Guide

Test your trading strategy on historical data to validate performance before risking real capital.

## What is Backtesting?

Backtesting simulates how your trading strategy would have performed using past trade data. It helps you:

- Validate strategy effectiveness
- Identify weaknesses
- Optimize parameters
- Build confidence before live trading

## How It Works

The backtesting engine:

1. Loads your historical trade data from `trade_history.json`
2. Simulates each trade with configured position sizes
3. Tracks portfolio performance over time
4. Calculates comprehensive metrics
5. Generates a detailed performance report

## Running a Backtest

### Prerequisites

You need historical trade data. Run the bot in paper trading mode first:

```bash
cargo run --bin bot-enhanced
```

Let it collect at least 10-20 trades for meaningful backtest results.

### Run the Backtest

```bash
cargo run --bin backtest
```

### Configure Backtest Parameters

Edit `.env` to customize:

```bash
BACKTEST_STARTING_CAPITAL=100  # How much you're starting with
BACKTEST_POSITION_SIZE=10      # How much per trade
```

## Understanding the Results

### Performance Metrics

**Starting/Ending Capital**: Your portfolio value at start and end
**Total PnL**: Total profit or loss
**ROI**: Return on investment percentage

### Trade Statistics

**Total Trades**: Number of trades executed
**Win Rate**: Percentage of winning trades
**Avg Win/Loss**: Average profit per win/loss

### Risk Metrics

**Max Drawdown**: Largest peak-to-trough decline
- Lower is better (< 20% is good)
- Shows how much capital you could lose

**Sharpe Ratio**: Risk-adjusted return
- Higher is better (> 1.0 is good, > 2.0 is excellent)
- Measures return relative to volatility

**Profit Factor**: Gross profit / Gross loss
- Higher is better (> 1.5 is good)
- Shows how much you make per dollar lost

### Strategy Rating

The backtester rates your strategy based on:
- ⭐⭐⭐⭐⭐ EXCELLENT: All metrics are strong
- ⭐⭐⭐⭐ GOOD: Most metrics are strong
- ⭐⭐⭐ AVERAGE: Mixed performance
- ⭐⭐ BELOW AVERAGE: Needs improvement
- ⭐ NEEDS IMPROVEMENT: Poor performance

## Example Output

```
╔═══════════════════════════════════════════════════════════╗
║                    BACKTEST RESULTS                        ║
╠═══════════════════════════════════════════════════════════╣
║ 💰 PERFORMANCE
║   Starting Capital: $100.00
║   Ending Capital:   $145.50
║   Total PnL:        $45.50
║   ROI:              45.50%
║
║ 📊 TRADE STATISTICS
║   Total Trades:     25
║   Winning Trades:   18
║   Losing Trades:    7
║   Win Rate:         72.0%
║
║ 💵 WIN/LOSS ANALYSIS
║   Average Win:      $4.20
║   Average Loss:     $-2.10
║   Biggest Win:      $15.50
║   Biggest Loss:     $-5.00
║   Profit Factor:    2.45
║
║ 📈 RISK METRICS
║   Max Drawdown:     12.5%
║   Sharpe Ratio:     1.85
║   Avg Hold Time:    145 min
╠═══════════════════════════════════════════════════════════╣
║ 🎯 STRATEGY RATING: ⭐⭐⭐⭐⭐ EXCELLENT
╚═══════════════════════════════════════════════════════════╝
```

## Interpreting Results

### Good Strategy Indicators ✅

- Win rate > 60%
- ROI > 20%
- Profit factor > 1.5
- Sharpe ratio > 1.0
- Max drawdown < 20%

### Red Flags 🚩

- Win rate < 50% (coin flip is better!)
- Negative ROI (losing money)
- Profit factor < 1.0 (loses more than wins)
- Max drawdown > 30% (too risky)
- Sharpe ratio < 0.5 (poor risk-adjusted returns)

## Optimization Tips

### If Win Rate is Low:
- Increase MIN_SMART_MONEY_SCORE (be more selective)
- Add more filters to signal detection
- Review chart analysis parameters

### If Profit Factor is Low:
- Adjust stop-loss (maybe too tight)
- Adjust take-profit (maybe too greedy)
- Review position sizing

### If Max Drawdown is High:
- Reduce position size
- Limit concurrent positions
- Implement better risk management

### If Sharpe Ratio is Low:
- Strategy is too volatile
- Consider more stable entry signals
- Reduce position sizes

## Saved Results

Results are automatically saved to `backtest_results.json` for later analysis.

## Advanced: Comparing Strategies

Run multiple backtests with different parameters:

```bash
# Conservative strategy
BACKTEST_POSITION_SIZE=5 cargo run --bin backtest

# Aggressive strategy  
BACKTEST_POSITION_SIZE=20 cargo run --bin backtest
```

Compare the results to find your optimal risk level.

## Limitations

**Historical Performance ≠ Future Results**

Backtesting shows what *would have* happened, not what *will* happen.

- Markets change
- Past patterns may not repeat
- Real trading has slippage, fees
- Psychological factors aren't simulated

Always:
- Start with paper trading
- Use small position sizes initially
- Monitor performance continuously
- Be prepared to adjust

## Next Steps

1. Run backtest on your historical data
2. Analyze the results
3. Optimize parameters if needed
4. Test optimizations in paper trading
5. Gradually transition to live trading with small sizes

Remember: A good backtest is encouraging, but real-world validation is essential!
