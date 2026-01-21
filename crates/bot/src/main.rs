use anyhow::Result;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use tokio;
use tracing::{info, error, warn};
use solana_sdk::pubkey::Pubkey;

use trading_core::*;
use trading_data::*;
use trading_analysis::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "info".to_string()),
        )
        .init();

    // Load environment
    dotenvy::dotenv().ok();

    info!("🤖 Solana Smart Money Trading Bot - FREE TIER");
    info!("============================================");

    // Check if trading is enabled
    let trading_enabled = std::env::var("TRADING_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    if trading_enabled {
        warn!("⚠️  LIVE TRADING ENABLED");
    } else {
        info!("📝 PAPER TRADING MODE (Safe)");
    }

    // Initialize RPC client
    info!("🔌 Connecting to Solana RPC...");
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

    let fallback_rpcs = vec![
        std::env::var("SOLANA_FALLBACK_RPC_1")
            .unwrap_or_else(|_| "https://solana-api.projectserum.com".to_string()),
        std::env::var("SOLANA_FALLBACK_RPC_2")
            .unwrap_or_else(|_| "https://rpc.ankr.com/solana".to_string()),
    ];

    let rpc_client = FallbackRpcClient::new(
        rpc_url.clone(),
        fallback_rpcs,
        solana_sdk::commitment_config::CommitmentConfig::confirmed(),
    );

    // Test RPC connection
    match rpc_client.get_slot().await {
        Ok(slot) => info!("✅ RPC connected! Current slot: {}", slot),
        Err(e) => {
            error!("❌ Failed to connect to RPC: {}", e);
            return Err(e.into());
        }
    }

    // Initialize data fetchers
    let dexscreener_url = std::env::var("DEXSCREENER_API_URL")
        .unwrap_or_else(|_| "https://api.dexscreener.com/latest".to_string());
    let token_fetcher = TokenDataFetcher::new(dexscreener_url);

    let rugcheck_url = std::env::var("RUGCHECK_API_URL")
        .unwrap_or_else(|_| "https://api.rugcheck.xyz/v1".to_string());
    let scam_detector = ScamDetector::new(rugcheck_url);

    // Load tracked wallets
    info!("📂 Loading tracked wallets...");
    let tracked_wallets = load_tracked_wallets("tracked_wallets.txt")?;

    if tracked_wallets.is_empty() {
        warn!("⚠️  No wallets to track!");
        warn!("   Add wallet addresses to tracked_wallets.txt");
        warn!("   Find them on dexscreener.com or photon");
        return Ok(());
    }

    info!("📊 Tracking {} smart wallets", tracked_wallets.len());

    // Configuration
    let min_smart_score = std::env::var("MIN_SMART_MONEY_SCORE")
        .unwrap_or_else(|_| "0.8".to_string())
        .parse::<f64>()
        .unwrap_or(0.8);

    let analysis_interval = std::env::var("WALLET_ANALYSIS_INTERVAL")
        .unwrap_or_else(|_| "600".to_string())
        .parse::<u64>()
        .unwrap_or(600);

    info!("⚙️  Configuration:");
    info!("   Min Smart Money Score: {:.2}", min_smart_score);
    info!("   Analysis Interval: {}s", analysis_interval);

    // Main bot loop
    info!("\n🚀 Starting main loop...\n");

    let mut cycle = 0;
    loop {
        cycle += 1;
        info!("🔄 Analysis Cycle #{}", cycle);
        info!("====================");

        let mut wallet_analyses = HashMap::new();

        // Analyze each tracked wallet
        for (idx, wallet) in tracked_wallets.iter().enumerate() {
            info!("[{}/{}] Analyzing {}...", idx + 1, tracked_wallets.len(), wallet);

            match analyze_wallet(&rpc_client, wallet).await {
                Ok(analysis) => {
                    info!(
                        "   ✅ Score: {:.2} | Win Rate: {:.1}% | Trades: {}",
                        analysis.smart_money_score,
                        analysis.metrics.win_rate,
                        analysis.metrics.total_trades,
                    );

                    if analysis.smart_money_score >= min_smart_score {
                        wallet_analyses.insert(*wallet, analysis);
                    } else {
                        warn!("   ⚠️  Score too low, skipping");
                    }
                }
                Err(e) => {
                    error!("   ❌ Error: {}", e);
                }
            }

            // Rate limiting (free tier friendly)
            tokio::time::sleep(Duration::from_secs(3)).await;
        }

        info!("\n✨ Found {} high-quality wallets", wallet_analyses.len());

        // Look for alpha signals
        if !wallet_analyses.is_empty() {
            info!("\n🔍 Scanning for alpha signals...");

            let signals = detect_alpha_signals(&wallet_analyses).await;

            if signals.is_empty() {
                info!("   No signals detected this cycle");
            } else {
                info!("   🎯 Found {} signals!", signals.len());

                for signal in signals {
                    info!("\n   ┌─ SIGNAL ─────────────────────");
                    info!("   │ Token: {}", signal.token_mint);
                    info!("   │ Type: {:?}", signal.signal_type);
                    info!("   │ Confidence: {:.1}%", signal.confidence * 100.0);
                    info!("   │ Reason: {}", signal.reason);
                    info!("   └──────────────────────────────");

                    if signal.confidence > 0.7 {
                        // Check token safety
                        info!("   🛡️  Checking token security...");

                        match scam_detector.check_token_security(&signal.token_mint).await {
                            Ok(security) => {
                                if security.is_scam {
                                    error!("   ❌ SCAM DETECTED! Skipping.");
                                    continue;
                                }

                                if security.is_bundle {
                                    warn!("   ⚠️  Bundle detected, skipping.");
                                    continue;
                                }

                                info!("   ✅ Security check passed");

                                // Get token data
                                match token_fetcher.get_token_data(&signal.token_mint).await {
                                    Ok(token) => {
                                        info!("   💰 Price: ${}", token.market_data.price_usd);
                                        info!("   💧 Liquidity: ${}", token.market_data.liquidity_usd);

                                        // Check if worth trading (free tier checks)
                                        if is_worth_trading(&token) {
                                            if trading_enabled {
                                                info!("   🔥 EXECUTING TRADE (would execute)");
                                                // TODO: Implement actual execution
                                            } else {
                                                info!("   📝 [PAPER] Would execute trade");
                                            }
                                        } else {
                                            warn!("   ⚠️  Token doesn't meet criteria");
                                        }
                                    }
                                    Err(e) => {
                                        error!("   ❌ Failed to fetch token: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("   ⚠️  Security check failed: {}", e);
                            }
                        }
                    }
                }
            }
        }

        // Sleep until next cycle
        info!("\n💤 Sleeping for {} seconds...\n", analysis_interval);
        tokio::time::sleep(Duration::from_secs(analysis_interval)).await;
    }
}

fn load_tracked_wallets(path: &str) -> Result<Vec<Pubkey>> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => {
            // Create empty file if it doesn't exist
            std::fs::write(path, "# Add wallet addresses here, one per line\n")?;
            return Ok(vec![]);
        }
    };

    let reader = BufReader::new(file);
    let mut wallets = vec![];

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();

        // Skip comments and empty lines
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
) -> Result<WalletAnalysis> {
    // Get recent trades (limit to save on RPC calls)
    let trades = TransactionParser::get_wallet_trades(rpc, wallet, 50).await?;

    if trades.is_empty() {
        return Err(anyhow::anyhow!("No trades found"));
    }

    // Build analysis
    let analysis = WalletMetricsCalculator::build_wallet_analysis(wallet, &trades)?;

    Ok(analysis)
}

#[derive(Debug, Clone)]
struct AlphaSignal {
    token_mint: Pubkey,
    signal_type: AlphaType,
    confidence: f64,
    reason: String,
}

#[derive(Debug, Clone)]
enum AlphaType {
    SmartMoneyConvergence,
    TopWalletTrade,
    UnusualActivity,
}

async fn detect_alpha_signals(
    wallet_analyses: &HashMap<Pubkey, WalletAnalysis>,
) -> Vec<AlphaSignal> {
    let mut signals = vec![];

    // Simple alpha detection for free tier
    // Signal 1: Top wallet made recent trade
    for (_wallet, analysis) in wallet_analyses.iter() {
        if analysis.smart_money_score > 0.85
            && analysis.metrics.win_rate > 75.0
            && analysis.metrics.total_trades >= 20
        {
            // This is an elite wallet
            // In full version, would check their recent trades
            // For now, just placeholder
        }
    }

    // Signal 2: Multiple wallets trading same token
    // TODO: Implement when database is set up

    signals
}

fn is_worth_trading(token: &Token) -> bool {
    use rust_decimal::Decimal;

    // Free tier safety checks

    // 1. Minimum liquidity
    if token.market_data.liquidity_usd < Decimal::from(10000) {
        return false;
    }

    // 2. Minimum volume
    if token.market_data.volume_24h < Decimal::from(5000) {
        return false;
    }

    // 3. Market cap range
    let mc = token.market_data.market_cap;
    if mc < Decimal::from(50000) || mc > Decimal::from(100_000_000) {
        return false;
    }

    // 4. Not too volatile
    if token.market_data.price_change_1h.abs() > 50.0 {
        return false;
    }

    true
}
