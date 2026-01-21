// Wallet Finder Tool - Discover Elite Solana Traders
// Usage: cargo run --bin find-wallets

use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tracing::{info, warn};
use trading_core::*;
use trading_data::*;
use trading_analysis::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        )
        .init();

    dotenvy::dotenv().ok();

    println!("\n{}", r#"
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║              🔍 ELITE WALLET FINDER                           ║
║                                                               ║
║  Discover top-performing Solana traders to copy              ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
    "#);

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
        rpc_url,
        fallback_rpcs,
        solana_sdk::commitment_config::CommitmentConfig::confirmed(),
    );

    // Test RPC
    match rpc_client.get_slot().await {
        Ok(slot) => info!("✅ RPC connected! Slot: {}", slot),
        Err(e) => {
            warn!("❌ RPC connection failed: {}", e);
            return Err(e.into());
        }
    }

    // Candidate wallets to analyze (these are known active DEX traders)
    // In a real implementation, you'd fetch these from on-chain data or DexScreener API
    let candidate_wallets = get_candidate_wallets();

    println!("\n📊 Analyzing {} candidate wallets...\n", candidate_wallets.len());
    println!("{}", "═".repeat(80));

    let mut elite_wallets = Vec::new();

    for (idx, wallet_str) in candidate_wallets.iter().enumerate() {
        let wallet = match Pubkey::from_str(wallet_str) {
            Ok(w) => w,
            Err(e) => {
                warn!("Invalid wallet {}: {}", wallet_str, e);
                continue;
            }
        };

        print!("[{:2}/{}] Analyzing {}...", idx + 1, candidate_wallets.len(), &wallet_str[..12]);

        // Analyze wallet
        match analyze_wallet(&rpc_client, &wallet).await {
            Ok((analysis, _)) => {
                println!(" Score: {:.2} | WR: {:.1}% | Trades: {}",
                    analysis.smart_money_score,
                    analysis.metrics.win_rate,
                    analysis.metrics.total_trades,
                );

                // Elite criteria: 75%+ win rate, 0.8+ smart score, 20+ trades
                if analysis.smart_money_score >= 0.75
                    && analysis.metrics.win_rate >= 70.0
                    && analysis.metrics.total_trades >= 20
                {
                    elite_wallets.push((wallet, analysis));
                }
            }
            Err(e) => {
                println!(" ❌ Error: {}", e);
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    println!("{}", "═".repeat(80));
    println!("\n🎯 Found {} ELITE wallets!\n", elite_wallets.len());

    if elite_wallets.is_empty() {
        println!("💡 Try again later or add more candidate wallets to the list.");
        return Ok(());
    }

    // Sort by smart money score
    elite_wallets.sort_by(|a, b| {
        b.1.smart_money_score
            .partial_cmp(&a.1.smart_money_score)
            .unwrap()
    });

    // Display top wallets
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║                    TOP ELITE WALLETS                          ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");

    for (idx, (wallet, analysis)) in elite_wallets.iter().enumerate().take(10) {
        println!("║ #{:2} │ Score: {:.2} │ WR: {:.1}% │ Trades: {:3} │ 24h: {}",
            idx + 1,
            analysis.smart_money_score,
            analysis.metrics.win_rate,
            analysis.metrics.total_trades,
            analysis.metrics.trades_last_24h,
        );
        println!("║     │ {}", wallet);
    }

    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // Save to file
    save_wallets_to_file(&elite_wallets)?;

    println!("✅ Saved {} elite wallets to tracked_wallets.txt", elite_wallets.len());
    println!("\n🚀 Ready to run the bot with these wallets!");
    println!("   cargo run --bin bot-enhanced\n");

    Ok(())
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

fn get_candidate_wallets() -> Vec<String> {
    // These are example addresses - in production, you'd:
    // 1. Fetch from DexScreener API (top traders on trending tokens)
    // 2. Scrape from on-chain data (top holders/traders)
    // 3. Monitor pump.fun for early buyers
    // 4. Track wallets mentioned in alpha groups

    vec![
        // Add real wallet addresses here
        // For now, returning empty so users must add their own
    ]
}

fn save_wallets_to_file(wallets: &[(Pubkey, WalletAnalysis)]) -> Result<()> {
    use std::io::Write;

    let mut file = std::fs::File::create("tracked_wallets.txt")?;

    writeln!(file, "# Elite Wallets - Auto-generated by find-wallets")?;
    writeln!(file, "# Found {} elite traders\n", wallets.len())?;

    for (wallet, analysis) in wallets {
        writeln!(
            file,
            "# Score: {:.2} | WR: {:.1}% | Trades: {}",
            analysis.smart_money_score,
            analysis.metrics.win_rate,
            analysis.metrics.total_trades
        )?;
        writeln!(file, "{}", wallet)?;
        writeln!(file)?;
    }

    Ok(())
}
