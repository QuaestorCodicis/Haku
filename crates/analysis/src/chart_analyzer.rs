use trading_core::*;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct ChartSignal {
    pub action: TradeAction,
    pub confidence: f64,
    pub reason: String,
    pub suggested_entry: Decimal,
    pub suggested_exit: Decimal,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TradeAction {
    StrongBuy,
    Buy,
    Hold,
    Sell,
    StrongSell,
}

pub struct ChartAnalyzer;

impl ChartAnalyzer {
    /// Analyze token using price action and volume
    pub fn analyze_entry_exit(token: &Token) -> ChartSignal {
        let market = &token.market_data;

        let price_5m = market.price_change_5m;
        let price_1h = market.price_change_1h;
        let price_24h = market.price_change_24h;

        let volume = market.volume_24h;
        let liquidity = market.liquidity_usd;

        // Pattern 1: Strong uptrend with volume
        if price_5m > 5.0 && price_1h > 10.0 && price_24h > 20.0 {
            if volume > liquidity * Decimal::from(2) {
                return ChartSignal {
                    action: TradeAction::StrongBuy,
                    confidence: 0.85,
                    reason: "Strong uptrend + volume breakout 🚀".into(),
                    suggested_entry: market.price_usd,
                    suggested_exit: market.price_usd * Decimal::from_f64_retain(1.5).unwrap(),
                };
            }
        }

        // Pattern 2: Dip buying opportunity
        if price_5m < -2.0 && price_5m > -5.0 && price_24h > 10.0 {
            return ChartSignal {
                action: TradeAction::Buy,
                confidence: 0.75,
                reason: "Healthy pullback in uptrend 📊".into(),
                suggested_entry: market.price_usd,
                suggested_exit: market.price_usd * Decimal::from_f64_retain(1.3).unwrap(),
            };
        }

        // Pattern 3: Breakout from consolidation
        if price_5m.abs() < 1.0 && price_1h.abs() < 2.0 {
            if volume > liquidity * Decimal::from_f64_retain(1.5).unwrap() {
                return ChartSignal {
                    action: TradeAction::Buy,
                    confidence: 0.7,
                    reason: "Consolidation breakout ⚡".into(),
                    suggested_entry: market.price_usd,
                    suggested_exit: market.price_usd * Decimal::from_f64_retain(1.25).unwrap(),
                };
            }
        }

        // Pattern 4: Early pump detection
        if price_5m > 8.0 && price_1h > 15.0 && price_24h < 30.0 {
            if volume > liquidity {
                return ChartSignal {
                    action: TradeAction::StrongBuy,
                    confidence: 0.8,
                    reason: "Early pump detected 🎯".into(),
                    suggested_entry: market.price_usd,
                    suggested_exit: market.price_usd * Decimal::from_f64_retain(1.4).unwrap(),
                };
            }
        }

        // Pattern 5: Overbought (take profit zone)
        if price_5m > 20.0 && price_1h > 50.0 {
            return ChartSignal {
                action: TradeAction::Sell,
                confidence: 0.8,
                reason: "Overbought - take profits ⚠️".into(),
                suggested_entry: Decimal::ZERO,
                suggested_exit: market.price_usd,
            };
        }

        // Pattern 6: Downtrend (exit)
        if price_5m < -5.0 && price_1h < -10.0 && price_24h < -15.0 {
            return ChartSignal {
                action: TradeAction::StrongSell,
                confidence: 0.9,
                reason: "Strong downtrend - exit 🚨".into(),
                suggested_entry: Decimal::ZERO,
                suggested_exit: market.price_usd,
            };
        }

        // Pattern 7: Volume spike (potential pump starting)
        if volume > liquidity * Decimal::from(3) && price_5m > 3.0 {
            return ChartSignal {
                action: TradeAction::Buy,
                confidence: 0.75,
                reason: "Unusual volume spike 💥".into(),
                suggested_entry: market.price_usd,
                suggested_exit: market.price_usd * Decimal::from_f64_retain(1.35).unwrap(),
            };
        }

        // Default: wait for better setup
        ChartSignal {
            action: TradeAction::Hold,
            confidence: 0.5,
            reason: "No clear pattern - waiting ⏳".into(),
            suggested_entry: market.price_usd,
            suggested_exit: market.price_usd * Decimal::from_f64_retain(1.2).unwrap(),
        }
    }

    /// Calculate RSI approximation
    pub fn calculate_rsi_approx(
        price_5m: f64,
        price_1h: f64,
        price_24h: f64,
    ) -> f64 {
        let gains = [
            if price_5m > 0.0 { price_5m } else { 0.0 },
            if price_1h > 0.0 { price_1h } else { 0.0 },
            if price_24h > 0.0 { price_24h } else { 0.0 },
        ];

        let losses = [
            if price_5m < 0.0 { -price_5m } else { 0.0 },
            if price_1h < 0.0 { -price_1h } else { 0.0 },
            if price_24h < 0.0 { -price_24h } else { 0.0 },
        ];

        let avg_gain = gains.iter().sum::<f64>() / gains.len() as f64;
        let avg_loss = losses.iter().sum::<f64>() / losses.len() as f64;

        if avg_loss == 0.0 {
            return 100.0;
        }

        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }

    /// Detect if price is at support/resistance
    pub fn is_at_support_resistance(
        current_price: Decimal,
        recent_prices: &[Decimal],
    ) -> (bool, bool) {
        if recent_prices.is_empty() {
            return (false, false);
        }

        let tolerance = Decimal::from_f64_retain(0.02).unwrap(); // 2%

        let at_support = recent_prices.iter().any(|&price| {
            let diff = (current_price - price).abs() / price;
            diff < tolerance && price < current_price
        });

        let at_resistance = recent_prices.iter().any(|&price| {
            let diff = (current_price - price).abs() / price;
            diff < tolerance && price > current_price
        });

        (at_support, at_resistance)
    }
}
