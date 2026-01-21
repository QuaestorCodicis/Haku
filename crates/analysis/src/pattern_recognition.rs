// Pattern recognition for trade analysis
use trading_core::{Result, Trade, TradePosition};

pub struct PatternRecognizer;

impl PatternRecognizer {
    /// Detect common trading patterns
    pub fn detect_patterns(positions: &[TradePosition]) -> Vec<String> {
        let mut patterns = Vec::new();

        // Scalping pattern (very short hold times)
        if Self::is_scalping_pattern(positions) {
            patterns.push("Scalping".to_string());
        }

        // Swing trading pattern (longer holds)
        if Self::is_swing_trading_pattern(positions) {
            patterns.push("Swing Trading".to_string());
        }

        // Momentum trading (buying on pumps)
        if Self::is_momentum_pattern(positions) {
            patterns.push("Momentum".to_string());
        }

        patterns
    }

    fn is_scalping_pattern(positions: &[TradePosition]) -> bool {
        let avg_hold = positions
            .iter()
            .filter_map(|p| p.hold_time_seconds)
            .sum::<f64>()
            / positions.len().max(1) as f64;

        avg_hold < 3600.0 // Less than 1 hour
    }

    fn is_swing_trading_pattern(positions: &[TradePosition]) -> bool {
        let avg_hold = positions
            .iter()
            .filter_map(|p| p.hold_time_seconds)
            .sum::<f64>()
            / positions.len().max(1) as f64;

        avg_hold > 86400.0 // More than 1 day
    }

    fn is_momentum_pattern(positions: &[TradePosition]) -> bool {
        // Check if most trades are buys during price increases
        // TODO: Implement with market data
        false
    }
}
