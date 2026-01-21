use reqwest::Client;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};
use trading_core::{MarketData, Result, Token, TokenMetadata, TradingError};
use rust_decimal::Decimal;
use chrono::Utc;

/// Token data fetcher using free APIs
pub struct TokenDataFetcher {
    client: Client,
    dexscreener_url: String,
    cache: Arc<RwLock<HashMap<Pubkey, (Token, i64)>>>,
    cache_ttl_seconds: i64,
}

#[derive(Debug, Deserialize)]
struct DexScreenerResponse {
    pairs: Option<Vec<DexScreenerPair>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DexScreenerPair {
    base_token: BaseToken,
    price_usd: Option<String>,
    liquidity: Option<Liquidity>,
    volume: Option<Volume>,
    price_change: Option<PriceChange>,
    info: Option<PairInfo>,
    dex_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BaseToken {
    address: String,
    name: String,
    symbol: String,
}

#[derive(Debug, Deserialize)]
struct Liquidity {
    usd: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct Volume {
    h24: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PriceChange {
    h24: Option<f64>,
    h1: Option<f64>,
    m5: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PairInfo {
    image_url: Option<String>,
    websites: Option<Vec<Website>>,
    socials: Option<Vec<Social>>,
}

#[derive(Debug, Deserialize)]
struct Website {
    url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Social {
    r#type: String,
    url: String,
}

impl TokenDataFetcher {
    pub fn new(dexscreener_url: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            dexscreener_url,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl_seconds: 60, // Cache for 1 minute
        }
    }

    /// Get token data from DexScreener
    pub async fn get_token_data(&self, mint: &Pubkey) -> Result<Token> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some((token, timestamp)) = cache.get(mint) {
                let now = Utc::now().timestamp();
                if now - timestamp < self.cache_ttl_seconds {
                    debug!("Cache hit for token {}", mint);
                    return Ok(token.clone());
                }
            }
        }

        // Fetch from API
        let url = format!("{}/dex/tokens/{}", self.dexscreener_url, mint);
        debug!("Fetching token data from DexScreener: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| TradingError::DataFetchError(format!("DexScreener request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(TradingError::DataFetchError(format!(
                "DexScreener returned status: {}",
                response.status()
            )));
        }

        let data: DexScreenerResponse = response
            .json()
            .await
            .map_err(|e| TradingError::ParseError(format!("Failed to parse DexScreener response: {}", e)))?;

        let pair = data
            .pairs
            .and_then(|pairs| pairs.into_iter().next())
            .ok_or_else(|| TradingError::DataFetchError("No pairs found for token".to_string()))?;

        let token = self.parse_dexscreener_data(mint, pair)?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(*mint, (token.clone(), Utc::now().timestamp()));
        }

        Ok(token)
    }

    /// Parse DexScreener data into Token struct
    fn parse_dexscreener_data(&self, mint: &Pubkey, pair: DexScreenerPair) -> Result<Token> {
        let price_usd = pair
            .price_usd
            .and_then(|p| Decimal::from_str(&p).ok())
            .unwrap_or(Decimal::ZERO);

        let liquidity_usd = pair
            .liquidity
            .and_then(|l| l.usd)
            .map(|v| Decimal::from_f64_retain(v).unwrap_or(Decimal::ZERO))
            .unwrap_or(Decimal::ZERO);

        let volume_24h = pair
            .volume
            .and_then(|v| v.h24)
            .map(|v| Decimal::from_f64_retain(v).unwrap_or(Decimal::ZERO))
            .unwrap_or(Decimal::ZERO);

        let price_change_24h = pair
            .price_change
            .as_ref()
            .and_then(|pc| pc.h24)
            .unwrap_or(0.0);

        let price_change_1h = pair
            .price_change
            .as_ref()
            .and_then(|pc| pc.h1)
            .unwrap_or(0.0);

        let price_change_5m = pair
            .price_change
            .as_ref()
            .and_then(|pc| pc.m5)
            .unwrap_or(0.0);

        // Calculate market cap (approximate)
        let market_cap = liquidity_usd * Decimal::from(2); // Simple approximation

        let mut metadata = TokenMetadata::default();
        if let Some(info) = pair.info {
            metadata.logo_url = info.image_url;
            metadata.website = info.websites.and_then(|w| w.first().map(|w| w.url.clone()));

            if let Some(socials) = info.socials {
                for social in socials {
                    match social.r#type.as_str() {
                        "twitter" => metadata.twitter = Some(social.url),
                        "telegram" => metadata.telegram = Some(social.url),
                        _ => {}
                    }
                }
            }
        }

        Ok(Token {
            mint: *mint,
            symbol: pair.base_token.symbol,
            name: pair.base_token.name,
            decimals: 9, // Most Solana tokens use 9 decimals
            metadata,
            security: Default::default(), // Will be filled by scam checker
            market_data: MarketData {
                price_usd,
                price_sol: Decimal::ZERO, // Will be calculated separately
                market_cap,
                liquidity_usd,
                volume_24h,
                price_change_24h,
                price_change_1h,
                price_change_5m,
                holders: None,
                dex: pair.dex_id,
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Get multiple tokens in batch (with rate limiting)
    pub async fn get_tokens_batch(&self, mints: &[Pubkey]) -> Vec<(Pubkey, Result<Token>)> {
        let mut results = Vec::new();

        for mint in mints {
            let result = self.get_token_data(mint).await;
            results.push((*mint, result));

            // Rate limiting - DexScreener free tier allows ~30 requests/minute
            tokio::time::sleep(Duration::from_millis(2000)).await;
        }

        results
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        debug!("Token cache cleared");
    }

    /// Get cache size
    pub async fn cache_size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Ignore in CI to avoid hitting API rate limits
    async fn test_fetch_token_data() {
        let fetcher = TokenDataFetcher::new("https://api.dexscreener.com/latest".to_string());

        // Test with a known Solana token (e.g., BONK)
        let bonk_mint = Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263")
            .unwrap();

        let token = fetcher.get_token_data(&bonk_mint).await;
        assert!(token.is_ok());

        let token = token.unwrap();
        println!("Token: {:?}", token);
        assert_eq!(token.mint, bonk_mint);
    }
}
