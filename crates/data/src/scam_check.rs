use reqwest::Client;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};
use trading_core::{Result, RiskLevel, SecurityInfo, TradingError};
use std::collections::HashMap;

/// Scam detection using rugcheck.xyz free API
pub struct ScamDetector {
    client: Client,
    rugcheck_url: String,
    cache: Arc<RwLock<HashMap<Pubkey, (SecurityInfo, i64)>>>,
    cache_ttl_seconds: i64,
}

#[derive(Debug, Deserialize)]
struct RugCheckResponse {
    status: String,
    #[serde(rename = "tokenMeta")]
    token_meta: Option<TokenMeta>,
    risks: Option<Vec<Risk>>,
    score: Option<u32>,
    #[serde(rename = "topHolders")]
    top_holders: Option<Vec<Holder>>,
    markets: Option<Vec<Market>>,
}

#[derive(Debug, Deserialize)]
struct TokenMeta {
    name: Option<String>,
    symbol: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Risk {
    name: String,
    description: String,
    level: String,
    score: u32,
}

#[derive(Debug, Deserialize)]
struct Holder {
    address: String,
    percentage: f64,
}

#[derive(Debug, Deserialize)]
struct Market {
    lp: Option<LiquidityPool>,
}

#[derive(Debug, Deserialize)]
struct LiquidityPool {
    #[serde(rename = "lpLockedPct")]
    locked_pct: Option<f64>,
    #[serde(rename = "lpLocked")]
    locked: Option<bool>,
}

impl ScamDetector {
    pub fn new(rugcheck_url: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            rugcheck_url,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl_seconds: 300, // Cache for 5 minutes
        }
    }

    /// Check if a token is a scam using rugcheck.xyz
    pub async fn check_token_security(&self, mint: &Pubkey) -> Result<SecurityInfo> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some((security_info, timestamp)) = cache.get(mint) {
                let now = chrono::Utc::now().timestamp();
                if now - timestamp < self.cache_ttl_seconds {
                    debug!("Cache hit for security check {}", mint);
                    return Ok(security_info.clone());
                }
            }
        }

        // Fetch from rugcheck API
        let url = format!("{}/tokens/{}/report", self.rugcheck_url, mint);
        debug!("Fetching security data from rugcheck: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| TradingError::DataFetchError(format!("Rugcheck request failed: {}", e)))?;

        if !response.status().is_success() {
            warn!("Rugcheck returned status: {}", response.status());
            // Return default security info if rugcheck fails
            return Ok(SecurityInfo::default());
        }

        let data: RugCheckResponse = response
            .json()
            .await
            .map_err(|e| TradingError::ParseError(format!("Failed to parse rugcheck response: {}", e)))?;

        let security_info = self.parse_rugcheck_data(data)?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(*mint, (security_info.clone(), chrono::Utc::now().timestamp()));
        }

        Ok(security_info)
    }

    /// Parse rugcheck data into SecurityInfo
    fn parse_rugcheck_data(&self, data: RugCheckResponse) -> Result<SecurityInfo> {
        let rugcheck_score = data.score.map(|s| s as f64 / 100.0);

        // Analyze risks
        let mut is_scam = false;
        let mut is_bundle = false;
        let mut risk_level = RiskLevel::Low;

        if let Some(risks) = data.risks {
            for risk in risks {
                match risk.level.as_str() {
                    "danger" | "critical" => {
                        is_scam = true;
                        risk_level = RiskLevel::Critical;
                    }
                    "warn" | "warning" => {
                        if risk_level == RiskLevel::Low {
                            risk_level = RiskLevel::Medium;
                        }
                    }
                    _ => {}
                }

                // Check for bundle-related risks
                if risk.name.to_lowercase().contains("bundle")
                    || risk.description.to_lowercase().contains("bundl")
                {
                    is_bundle = true;
                }
            }
        }

        // Check top holders concentration
        let top_holders_percentage = if let Some(holders) = data.top_holders {
            holders.iter().take(10).map(|h| h.percentage).sum::<f64>()
        } else {
            0.0
        };

        // If top 10 holders own > 80%, increase risk
        if top_holders_percentage > 80.0 {
            is_bundle = true;
            if risk_level < RiskLevel::High {
                risk_level = RiskLevel::High;
            }
        }

        // Check LP lock status
        let mut lp_locked = false;
        let mut lp_lock_duration = None;

        if let Some(markets) = data.markets {
            if let Some(market) = markets.first() {
                if let Some(lp) = &market.lp {
                    lp_locked = lp.locked.unwrap_or(false);
                    if lp_locked {
                        // If LP is locked, reduce risk level
                        if risk_level == RiskLevel::Medium {
                            risk_level = RiskLevel::Low;
                        }
                    }
                }
            }
        }

        // Determine final risk assessment
        if rugcheck_score.is_some() && rugcheck_score.unwrap() < 30.0 {
            is_scam = true;
            risk_level = RiskLevel::Critical;
        }

        Ok(SecurityInfo {
            is_scam,
            is_bundle,
            rugcheck_score,
            lp_locked,
            lp_lock_duration,
            mint_authority_disabled: false, // Would need to check on-chain
            freeze_authority_disabled: false, // Would need to check on-chain
            top_holders_percentage,
            risk_level,
        })
    }

    /// Batch check multiple tokens
    pub async fn check_tokens_batch(&self, mints: &[Pubkey]) -> Vec<(Pubkey, Result<SecurityInfo>)> {
        let mut results = Vec::new();

        for mint in mints {
            let result = self.check_token_security(mint).await;
            results.push((*mint, result));

            // Rate limiting - be conservative with free API
            tokio::time::sleep(Duration::from_millis(2000)).await;
        }

        results
    }

    /// Simple heuristic-based bundle detection (doesn't require API)
    pub async fn detect_bundle_heuristic(
        &self,
        rpc_client: &crate::rpc::FallbackRpcClient,
        token_mint: &Pubkey,
    ) -> Result<bool> {
        // Check if token has suspicious characteristics:
        // 1. Very high concentration in top holders
        // 2. Multiple related wallets with similar activity patterns
        // 3. Coordinated buying/selling

        // For now, return false - full implementation would require:
        // - Token account analysis
        // - Holder distribution check
        // - Wallet relationship graph analysis

        debug!("Bundle detection for {} - returning false (not implemented)", token_mint);
        Ok(false)
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        debug!("Security check cache cleared");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[tokio::test]
    #[ignore] // Ignore in CI to avoid hitting API rate limits
    async fn test_rugcheck() {
        let detector = ScamDetector::new("https://api.rugcheck.xyz/v1".to_string());

        // Test with a known token
        let bonk_mint = Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263")
            .unwrap();

        let security_info = detector.check_token_security(&bonk_mint).await;
        assert!(security_info.is_ok());

        let info = security_info.unwrap();
        println!("Security info: {:?}", info);
    }
}
