use anyhow::Result;
use rust_decimal::Decimal;
use std::path::Path;
use tracing_subscriber;

use trading_bot::backtester::{Backtester, BacktestConfig};
use trading_bot::persistence::TradeHistory;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        )
        .init();

    println!("\n{}", r#"
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║          ╔╗ ╔═╗╔═╗╦╔═╔╦╗╔═╗╔═╗╔╦╗  ╔═╗╔╗╔╔═╗╦╔╗╔╔═╗         ║
║          ╠╩╗╠═╣║  ╠╩╗ ║ ║╣ ╚═╗ ║   ║╣ ║║║║ ╦║║║║║╣          ║
║          ╚═╝╩ ╩╚═╝╩ ╩ ╩ ╚═╝╚═╝ ╩   ╚═╝╝╚╝╚═╝╩╝╚╝╚═╝         ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
    "#);

    // Load trade history
    let history_path = Path::new("trade_history.json");
    let trade_history = TradeHistory::load(history_path)?;

    println!("📊 Loaded {} historical trades\n", trade_history.get_total_trades());

    if trade_history.get_total_trades() == 0 {
        println!("❌ No historical trades found. Run the bot first to collect trade data.");
        return Ok(());
    }

    // Configure backtest
    let starting_capital = std::env::var("BACKTEST_STARTING_CAPITAL")
        .unwrap_or_else(|_| "100".to_string())
        .parse::<f64>()
        .unwrap_or(100.0);

    let position_size = std::env::var("BACKTEST_POSITION_SIZE")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<f64>()
        .unwrap_or(10.0);

    let config = BacktestConfig {
        starting_capital: Decimal::from_f64_retain(starting_capital).unwrap(),
        position_size: Decimal::from_f64_retain(position_size).unwrap(),
        max_positions: 5,
        stop_loss_pct: 10.0,
        take_profit_pct: 100.0,
    };

    println!("⚙️  BACKTEST CONFIGURATION");
    println!("   Starting Capital: ${:.2}", config.starting_capital);
    println!("   Position Size: ${:.2}", config.position_size);
    println!("   Max Positions: {}", config.max_positions);
    println!("   Stop Loss: {:.1}%", config.stop_loss_pct);
    println!("   Take Profit: {:.1}%\n", config.take_profit_pct);

    // Run backtest
    let backtester = Backtester::new(config);
    let results = backtester.run(&trade_history)?;

    // Print results
    results.print_report();

    // Save results
    let results_path = Path::new("backtest_results.json");
    results.save_to_file(results_path)?;

    println!("💾 Results saved to backtest_results.json\n");

    Ok(())
}
