// Enhanced bot with portfolio tracking, statistics, and chart analysis
// Use with: cargo run --bin bot-enhanced

use anyhow::Result;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use tokio;
use tracing::{info, error, warn};
use solana_sdk::pubkey::Pubkey;
use rust_decimal::Decimal;
use chrono::Utc;

use trading_core::*;
use trading_data::*;
use trading_analysis::*;

mod portfolio_monitor;
mod alpha_accelerator;
mod position_manager;
mod persistence;
mod telegram;
mod dashboard;

use portfolio_monitor::*;
use alpha_accelerator::*;
use position_manager::*;
use persistence::*;
use telegram::*;
use dashboard::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging with colors
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "info".to_string()),
        )
        .init();

    dotenvy::dotenv().ok();

    // ASCII art banner
    println!("\n{}", r#"
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║   ╔═╗╔═╗╦  ╔═╗╔╗╔╔═╗  ╔╦╗╦═╗╔═╗╔╦╗╦╔╗╔╔═╗  ╔╗ ╔═╗╔╦╗         ║
║   ╚═╗║ ║║  ╠═╣║║║╠═╣   ║ ╠╦╝╠═╣ ║║║║║║║ ╦  ╠╩╗║ ║ ║          ║
║   ╚═╝╚═╝╩═╝╩ ╩╝╚╝╩ ╩   ╩ ╩╚═╩ ╩═╩╝╩╝╚╝╚═╝  ╚═╝╚═╝ ╩          ║
║                                                               ║
║              ENHANCED FREE TIER - ACCELERATED MODE            ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
    "#);

    // Check trading mode
    let trading_enabled = std::env::var("TRADING_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    if trading_enabled {
        warn!("⚠️  LIVE TRADING ENABLED ⚠️");
    } else {
        info!("📝 PAPER TRADING MODE (Recommended for testing)");
    }

    // Initialize components
    info!("🔌 Initializing components...");

    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

    let fallback_rpcs = vec![
        std::env::var("SOLANA_FALLBACK_RPC_1")
            .unwrap_or_else(|_| "https://solana-api.projectserum.com".to_string()),
        std::env::var("SOLANA_FALLBACK_RPC_2")
            .unwrap_or_else(|_| "https://rpc.ankr.com/solana".to_string()),
    ];

    let rpc_client = FallbackRpcClient::new(
        rpc_url,
        fallback_rpcs,
        solana_sdk::commitment_config::CommitmentConfig::confirmed(),
    );

    // Test RPC
    match rpc_client.get_slot().await {
        Ok(slot) => info!("✅ RPC connected! Slot: {}", slot),
        Err(e) => {
            error!("❌ RPC connection failed: {}", e);
            return Err(e.into());
        }
    }

    // Initialize data fetchers
    let token_fetcher = TokenDataFetcher::new(
        std::env::var("DEXSCREENER_API_URL")
            .unwrap_or_else(|_| "https://api.dexscreener.com/latest".to_string())
    );

    let scam_detector = ScamDetector::new(
        std::env::var("RUGCHECK_API_URL")
            .unwrap_or_else(|_| "https://api.rugcheck.xyz/v1".to_string())
    );

    // Load tracked wallets
    let tracked_wallets = load_tracked_wallets("tracked_wallets.txt")?;

    if tracked_wallets.is_empty() {
        warn!("⚠️  No wallets tracked!");
        warn!("   Add addresses to tracked_wallets.txt");
        warn!("   Find elite wallets on dexscreener.com");
        return Ok(());
    }

    info!("📊 Tracking {} elite wallets", tracked_wallets.len());

    // Initialize portfolio monitor
    let starting_capital = std::env::var("MAX_POSITION_SIZE_USD")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<f64>()
        .unwrap_or(10.0);

    let mut portfolio = PortfolioMonitor::new(
        Decimal::from_f64_retain(starting_capital).unwrap()
    );

    // Initialize alpha detector
    let alpha_detector = AlphaAccelerator::new(
        3,  // 3+ wallets = strong signal
        60, // Last 60 minutes
    );

    // Initialize position manager
    let position_manager = PositionManager::new();

    // Initialize Telegram notifier
    let telegram_enabled = std::env::var("TELEGRAM_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    let telegram = if telegram_enabled {
        let token = std::env::var("TELEGRAM_BOT_TOKEN")
            .expect("TELEGRAM_BOT_TOKEN must be set when TELEGRAM_ENABLED=true");
        let chat_id = std::env::var("TELEGRAM_CHAT_ID")
            .expect("TELEGRAM_CHAT_ID must be set when TELEGRAM_ENABLED=true")
            .parse::<i64>()
            .expect("TELEGRAM_CHAT_ID must be a valid integer");

        info!("📱 Telegram notifications ENABLED");
        let notifier = TelegramNotifier::new(token, chat_id);

        // Test notification
        if let Err(e) = notifier.test_notification().await {
            warn!("Failed to send test notification: {}", e);
        }

        notifier
    } else {
        info!("📱 Telegram notifications DISABLED");
        TelegramNotifier::disabled()
    };

    // Send startup notification
    telegram.notify_bot_started(Decimal::from_f64_retain(starting_capital).unwrap()).await;

    // Load trade history
    let history_path = std::path::Path::new("trade_history.json");
    let mut trade_history = TradeHistory::load(history_path).unwrap_or_else(|_| {
        TradeHistory::new(Decimal::from_f64_retain(starting_capital).unwrap())
    });

    // Initialize web dashboard
    let dashboard_enabled = std::env::var("DASHBOARD_ENABLED")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    let dashboard_state = if dashboard_enabled {
        let dashboard_port = std::env::var("DASHBOARD_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .unwrap_or(3000);

        let dashboard = DashboardServer::new(dashboard_port, trade_history.clone());
        let dashboard_state = dashboard.get_state();

        // Start dashboard server in background
        tokio::spawn(async move {
            dashboard.start().await;
        });

        info!("🌐 Web dashboard enabled at http://localhost:{}", dashboard_port);
        Some(dashboard_state)
    } else {
        info!("🌐 Web dashboard disabled");
        None
    };

    // Configuration
    let min_smart_score = std::env::var("MIN_SMART_MONEY_SCORE")
        .unwrap_or_else(|_| "0.8".to_string())
        .parse::<f64>()
        .unwrap_or(0.8);

    let analysis_interval = std::env::var("WALLET_ANALYSIS_INTERVAL")
        .unwrap_or_else(|_| "300".to_string())  // 5 minutes for faster updates
        .parse::<u64>()
        .unwrap_or(300);

    info!("\n⚙️  Configuration:");
    info!("   Min Smart Money Score: {:.2}", min_smart_score);
    info!("   Analysis Interval: {}s", analysis_interval);
    info!("   Starting Capital: ${:.2}", starting_capital);

    // Main loop
    info!("\n🚀 Starting accelerated trading loop...\n");

    let mut cycle = 0;
    let mut wallet_analyses: HashMap<Pubkey, WalletAnalysis> = HashMap::new();
    let mut all_trades: HashMap<Pubkey, Vec<Trade>> = HashMap::new();

    loop {
        cycle += 1;

        println!("\n{}", "═".repeat(70));
        info!("📊 CYCLE #{} - {}", cycle, Utc::now().format("%Y-%m-%d %H:%M:%S"));
        println!("{}", "═".repeat(70));

        // Analyze wallets
        info!("\n🔍 Analyzing {} wallets...", tracked_wallets.len());

        for (idx, wallet) in tracked_wallets.iter().enumerate() {
            match analyze_wallet(&rpc_client, wallet).await {
                Ok((analysis, trades)) => {
                    info!("[{:2}/{}] {} - Score: {:.2} | WR: {:.1}% | Trades: {}",
                        idx + 1,
                        tracked_wallets.len(),
                        &wallet.to_string()[..8],
                        analysis.smart_money_score,
                        analysis.metrics.win_rate,
                        analysis.metrics.total_trades,
                    );

                    if analysis.smart_money_score >= min_smart_score {
                        wallet_analyses.insert(*wallet, analysis);
                        all_trades.insert(*wallet, trades);
                    }
                }
                Err(e) => {
                    warn!("[{:2}/{}] {} - Error: {}",
                        idx + 1,
                        tracked_wallets.len(),
                        &wallet.to_string()[..8],
                        e
                    );
                }
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        info!("\n✅ Found {} high-quality wallets", wallet_analyses.len());

        // ULTRA ALPHA DETECTION
        if !wallet_analyses.is_empty() {
            info!("\n🎯 Scanning for ULTRA-HIGH confidence signals...");

            let ultra_signals = alpha_detector
                .find_ultra_high_confidence_signals(&wallet_analyses, &all_trades)
                .await;

            if ultra_signals.is_empty() {
                info!("   No ultra signals this cycle");
            } else {
                info!("   🔥 Found {} ULTRA signals!", ultra_signals.len());

                for (idx, signal) in ultra_signals.iter().enumerate() {
                    println!("\n╔════════════════════════════════════════════════════════╗");
                    println!("║ ULTRA SIGNAL #{} - CONFIDENCE: {:.0}%                  ", idx + 1, signal.confidence * 100.0);
                    println!("╠════════════════════════════════════════════════════════╣");
                    println!("║ Token: {}", signal.token_mint);
                    println!("║ Smart Wallets Buying: {} 🔥", signal.smart_wallets_count);
                    println!("║ Avg Smart Score: {:.2}", signal.avg_smart_score);
                    println!("║ Total Volume: ${:.2}", signal.total_volume);
                    println!("╚════════════════════════════════════════════════════════╝");

                    // Notify about ultra signal
                    telegram.notify_ultra_signal(
                        &signal.token_mint.to_string(),
                        signal.confidence,
                        signal.smart_wallets_count as usize
                    ).await;

                    if signal.confidence > 0.85 {
                        // Fetch token data
                        info!("   📊 Analyzing token...");

                        match token_fetcher.get_token_data(&signal.token_mint).await {
                            Ok(token) => {
                                // Check security
                                match scam_detector.check_token_security(&signal.token_mint).await {
                                    Ok(security) => {
                                        if security.is_scam {
                                            error!("   ❌ SCAM DETECTED! Skipping.");
                                            telegram.notify_scam_detected(&signal.token_mint.to_string()).await;
                                            continue;
                                        }

                                        if security.is_bundle {
                                            warn!("   ⚠️  Bundle detected. Skipping.");
                                            continue;
                                        }

                                        info!("   ✅ Security check passed");

                                        // CHART ANALYSIS for optimal entry
                                        let chart_signal = ChartAnalyzer::analyze_entry_exit(&token);

                                        info!("   📈 Chart Analysis:");
                                        info!("      Action: {:?}", chart_signal.action);
                                        info!("      Confidence: {:.0}%", chart_signal.confidence * 100.0);
                                        info!("      Reason: {}", chart_signal.reason);

                                        match chart_signal.action {
                                            TradeAction::StrongBuy | TradeAction::Buy => {
                                                info!("   💰 Price: ${}", token.market_data.price_usd);
                                                info!("   💧 Liquidity: ${}", token.market_data.liquidity_usd);
                                                info!("   📊 24h Volume: ${}", token.market_data.volume_24h);

                                                // Combined confidence
                                                let combined_confidence = (signal.confidence + chart_signal.confidence) / 2.0;

                                                if combined_confidence > 0.75 {
                                                    info!("\n   🚀 EXECUTING TRADE (Combined Confidence: {:.0}%)",
                                                        combined_confidence * 100.0);

                                                    if trading_enabled {
                                                        // TODO: Execute real trade
                                                        info!("   [LIVE] Would execute trade here");
                                                    } else {
                                                        info!("   [PAPER] Simulated buy at ${}", token.market_data.price_usd);

                                                        // Track in portfolio
                                                        let position = OpenPosition {
                                                            token_mint: signal.token_mint,
                                                            token_symbol: token.symbol.clone(),
                                                            entry_time: Utc::now(),
                                                            entry_price: token.market_data.price_usd,
                                                            entry_mc: token.market_data.market_cap,
                                                            amount: Decimal::from_f64_retain(starting_capital).unwrap(),
                                                            current_price: token.market_data.price_usd,
                                                            current_mc: token.market_data.market_cap,
                                                            unrealized_pnl: Decimal::ZERO,
                                                            unrealized_pnl_pct: 0.0,
                                                            stop_loss: chart_signal.suggested_entry * Decimal::from_f64_retain(0.9).unwrap(),
                                                            take_profit: chart_signal.suggested_exit,
                                                            hold_time_minutes: 0,
                                                        };

                                                        portfolio.open_position(position.clone());

                                                        // Send Telegram notification
                                                        telegram.notify_position_opened(&position, combined_confidence).await;
                                                    }
                                                }
                                            }
                                            TradeAction::Sell | TradeAction::StrongSell => {
                                                warn!("   ⚠️  Chart shows SELL signal, skipping buy");
                                            }
                                            _ => {}
                                        }
                                    }
                                    Err(e) => warn!("   ⚠️  Security check failed: {}", e),
                                }
                            }
                            Err(e) => error!("   ❌ Failed to fetch token: {}", e),
                        }
                    }
                }
            }

            // Check hot wallets (on winning streak)
            let hot_wallets = alpha_detector.find_hot_wallets(&wallet_analyses);
            if !hot_wallets.is_empty() {
                info!("\n🔥 {} wallets are HOT (on winning streak)!", hot_wallets.len());
                for wallet in hot_wallets.iter().take(3) {
                    info!("   💎 {}", wallet);
                }
            }
        }

        // Update and check open positions
        if let Err(e) = position_manager.check_and_update_positions(&mut portfolio, &token_fetcher).await {
            warn!("Failed to update positions: {}", e);
        }

        // Save trade history if there was a new closed trade
        if let Some(closed_trade) = portfolio.get_last_closed_trade() {
            // Send Telegram notification
            telegram.notify_position_closed(closed_trade).await;

            trade_history.add_closed_trade(closed_trade);
            trade_history.update_daily_stats(portfolio.get_daily_stats());

            if let Err(e) = trade_history.save(history_path) {
                warn!("Failed to save trade history: {}", e);
            }

            // Update dashboard
            if let Some(ref state) = dashboard_state {
                state.update_trade_history(&trade_history).await;
            }
        }

        // Update dashboard stats
        if let Some(ref state) = dashboard_state {
            state.update_stats(portfolio.get_daily_stats()).await;
        }

        // Display dashboard
        portfolio.print_dashboard();

        // Send periodic portfolio update (every 10 cycles or ~50 minutes with 5 min interval)
        if cycle % 10 == 0 {
            telegram.notify_portfolio_update(portfolio.get_daily_stats()).await;
        }

        // Sleep
        info!("\n💤 Next cycle in {} seconds...\n", analysis_interval);
        tokio::time::sleep(Duration::from_secs(analysis_interval)).await;
    }
}

fn load_tracked_wallets(path: &str) -> Result<Vec<Pubkey>> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => {
            std::fs::write(path, "# Add wallet addresses here, one per line\n")?;
            return Ok(vec![]);
        }
    };

    let reader = BufReader::new(file);
    let mut wallets = vec![];

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        match Pubkey::from_str(trimmed) {
            Ok(pubkey) => wallets.push(pubkey),
            Err(e) => eprintln!("⚠️  Invalid address '{}': {}", trimmed, e),
        }
    }

    Ok(wallets)
}

async fn analyze_wallet(
    rpc: &FallbackRpcClient,
    wallet: &Pubkey,
) -> Result<(WalletAnalysis, Vec<Trade>)> {
    let trades = TransactionParser::get_wallet_trades(rpc, wallet, 50).await?;

    if trades.is_empty() {
        return Err(anyhow::anyhow!("No trades found"));
    }

    let analysis = WalletMetricsCalculator::build_wallet_analysis(wallet, &trades)?;

    Ok((analysis, trades))
}
