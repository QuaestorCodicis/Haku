use std::collections::HashMap;
use solana_sdk::pubkey::Pubkey;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use trading_core::*;
use trading_analysis::*;

pub struct AlphaAccelerator {
    convergence_threshold: u32,  // How many wallets = strong signal
    time_window_minutes: i64,     // How recent
}

impl AlphaAccelerator {
    pub fn new(convergence_threshold: u32, time_window_minutes: i64) -> Self {
        Self {
            convergence_threshold,
            time_window_minutes,
        }
    }

    pub async fn find_ultra_high_confidence_signals(
        &self,
        wallets: &HashMap<Pubkey, WalletAnalysis>,
        recent_trades: &HashMap<Pubkey, Vec<Trade>>,
    ) -> Vec<UltraSignal> {
        let mut token_activity: HashMap<Pubkey, TokenActivity> = HashMap::new();

        // Scan last hour of activity
        let cutoff = Utc::now() - chrono::Duration::minutes(self.time_window_minutes);

        for (wallet, trades) in recent_trades {
            // Only elite wallets (80%+ win rate)
            if let Some(analysis) = wallets.get(wallet) {
                if analysis.smart_money_score < 0.8 {
                    continue;
                }

                for trade in trades {
                    if trade.timestamp < cutoff {
                        continue;
                    }

                    if trade.side != TradeSide::Buy {
                        continue;
                    }

                    let activity = token_activity
                        .entry(trade.token_mint)
                        .or_insert_with(|| TokenActivity::new(trade.token_mint));

                    activity.smart_wallets_bought.push(*wallet);
                    activity.total_volume += trade.amount_in;
                    activity.avg_smart_score += analysis.smart_money_score;
                }
            }
        }

        // Find convergence signals
        let mut ultra_signals = vec![];

        for (token, activity) in token_activity {
            let wallet_count = activity.smart_wallets_bought.len() as u32;

            if wallet_count >= self.convergence_threshold {
                let avg_score = activity.avg_smart_score / wallet_count as f64;

                ultra_signals.push(UltraSignal {
                    token_mint: token,
                    confidence: 0.8 + (wallet_count as f64 * 0.05).min(0.2),
                    smart_wallets_count: wallet_count,
                    avg_smart_score: avg_score,
                    total_volume: activity.total_volume,
                    signal_type: SignalType::SmartMoneyConvergence,
                    detected_at: Utc::now(),
                });
            }
        }

        // Sort by confidence
        ultra_signals.sort_by(|a, b| {
            b.confidence.partial_cmp(&a.confidence).unwrap()
        });

        ultra_signals
    }

    /// Detect wallets that are on a winning streak (HOT)
    pub fn find_hot_wallets(
        &self,
        wallets: &HashMap<Pubkey, WalletAnalysis>,
    ) -> Vec<Pubkey> {
        let mut hot_wallets = vec![];

        for (wallet, analysis) in wallets {
            // Hot = recent wins + high overall score
            if analysis.metrics.trades_last_24h >= 3
                && analysis.metrics.win_rate > 80.0
                && analysis.smart_money_score > 0.85
            {
                hot_wallets.push(*wallet);
            }
        }

        hot_wallets
    }
}

#[derive(Debug, Clone)]
struct TokenActivity {
    smart_wallets_bought: Vec<Pubkey>,
    total_volume: Decimal,
    avg_smart_score: f64,
}

impl TokenActivity {
    fn new(_token_mint: Pubkey) -> Self {
        Self {
            smart_wallets_bought: vec![],
            total_volume: Decimal::ZERO,
            avg_smart_score: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UltraSignal {
    pub token_mint: Pubkey,
    pub confidence: f64,
    pub smart_wallets_count: u32,
    pub avg_smart_score: f64,
    pub total_volume: Decimal,
    pub signal_type: SignalType,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum SignalType {
    SmartMoneyConvergence,  // 3+ wallets bought
    HotWalletTrade,         // Wallet on streak
    VolumeBreakout,         // Unusual volume
    ChartPattern,           // Technical pattern
}
