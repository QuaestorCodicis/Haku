use reqwest::Client;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::time::Duration;
use tracing::{debug, error};
use trading_core::{Result, TradingError};
use rust_decimal::Decimal;

/// Jupiter API client for swap quotes and routing
pub struct JupiterClient {
    client: Client,
    api_url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct QuoteRequest {
    input_mint: String,
    output_mint: String,
    amount: String,
    slippage_bps: u16,
    only_direct_routes: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    pub input_mint: String,
    pub in_amount: String,
    pub output_mint: String,
    pub out_amount: String,
    pub other_amount_threshold: String,
    pub swap_mode: String,
    pub slippage_bps: u16,
    pub price_impact_pct: f64,
    pub route_plan: Vec<RoutePlan>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutePlan {
    pub swap_info: SwapInfo,
    pub percent: u8,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapInfo {
    pub amm_key: String,
    pub label: Option<String>,
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub fee_amount: String,
    pub fee_mint: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SwapRequest {
    quote_response: QuoteResponse,
    user_public_key: String,
    wrap_unwrap_sol: bool,
    use_shared_accounts: bool,
    fee_account: Option<String>,
    prioritization_fee_lamports: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapResponse {
    pub swap_transaction: String,
    pub last_valid_block_height: u64,
}

impl JupiterClient {
    pub fn new(api_url: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .expect("Failed to create HTTP client"),
            api_url,
        }
    }

    /// Get swap quote from Jupiter
    pub async fn get_quote(
        &self,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount: u64,
        slippage_bps: u16,
    ) -> Result<QuoteResponse> {
        let url = format!("{}/quote", self.api_url);

        debug!(
            "Getting Jupiter quote: {} {} for {}",
            amount, input_mint, output_mint
        );

        let response = self
            .client
            .get(&url)
            .query(&[
                ("inputMint", input_mint.to_string()),
                ("outputMint", output_mint.to_string()),
                ("amount", amount.to_string()),
                ("slippageBps", slippage_bps.to_string()),
                ("onlyDirectRoutes", "false".to_string()),
            ])
            .send()
            .await
            .map_err(|e| TradingError::DataFetchError(format!("Jupiter quote request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TradingError::DataFetchError(format!(
                "Jupiter quote failed with status {}: {}",
                status, body
            )));
        }

        let quote: QuoteResponse = response
            .json()
            .await
            .map_err(|e| TradingError::ParseError(format!("Failed to parse Jupiter quote: {}", e)))?;

        debug!(
            "Quote received: {} in -> {} out (price impact: {}%)",
            quote.in_amount, quote.out_amount, quote.price_impact_pct
        );

        Ok(quote)
    }

    /// Get swap transaction from Jupiter
    pub async fn get_swap_transaction(
        &self,
        quote: QuoteResponse,
        user_pubkey: &Pubkey,
        priority_fee_lamports: Option<u64>,
    ) -> Result<SwapResponse> {
        let url = format!("{}/swap", self.api_url);

        debug!("Getting swap transaction for user {}", user_pubkey);

        let request = SwapRequest {
            quote_response: quote,
            user_public_key: user_pubkey.to_string(),
            wrap_unwrap_sol: true,
            use_shared_accounts: true,
            fee_account: None,
            prioritization_fee_lamports: priority_fee_lamports,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| TradingError::ExecutionError(format!("Jupiter swap request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TradingError::ExecutionError(format!(
                "Jupiter swap failed with status {}: {}",
                status, body
            )));
        }

        let swap: SwapResponse = response
            .json()
            .await
            .map_err(|e| TradingError::ParseError(format!("Failed to parse Jupiter swap: {}", e)))?;

        debug!("Swap transaction received");

        Ok(swap)
    }

    /// Get price for a token pair (1 unit of input)
    pub async fn get_price(
        &self,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        decimals: u8,
    ) -> Result<Decimal> {
        let amount = 10_u64.pow(decimals as u32);

        let quote = self.get_quote(input_mint, output_mint, amount, 100).await?;

        let in_amount = Decimal::from_str(&quote.in_amount)
            .map_err(|e| TradingError::ParseError(format!("Invalid in_amount: {}", e)))?;

        let out_amount = Decimal::from_str(&quote.out_amount)
            .map_err(|e| TradingError::ParseError(format!("Invalid out_amount: {}", e)))?;

        if in_amount == Decimal::ZERO {
            return Err(TradingError::ParseError("Input amount is zero".to_string()));
        }

        let price = out_amount / in_amount;

        Ok(price)
    }

    /// Get SOL/USDC price
    pub async fn get_sol_price_usd(&self) -> Result<Decimal> {
        let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")
            .map_err(|e| TradingError::ParseError(format!("Invalid SOL mint: {}", e)))?;

        let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
            .map_err(|e| TradingError::ParseError(format!("Invalid USDC mint: {}", e)))?;

        self.get_price(&sol_mint, &usdc_mint, 9).await
    }

    /// Calculate expected output amount
    pub fn calculate_output_amount(
        &self,
        input_amount: Decimal,
        price: Decimal,
    ) -> Decimal {
        input_amount * price
    }

    /// Calculate price impact percentage
    pub fn calculate_price_impact(
        &self,
        expected_price: Decimal,
        actual_price: Decimal,
    ) -> f64 {
        if expected_price == Decimal::ZERO {
            return 0.0;
        }

        let impact = ((actual_price - expected_price) / expected_price) * Decimal::from(100);
        impact.to_string().parse::<f64>().unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Ignore in CI to avoid hitting API rate limits
    async fn test_jupiter_quote() {
        let client = JupiterClient::new("https://quote-api.jup.ag/v6".to_string());

        let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
        let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();

        let amount = 1_000_000_000; // 1 SOL

        let quote = client.get_quote(&sol_mint, &usdc_mint, amount, 100).await;
        assert!(quote.is_ok());

        let quote = quote.unwrap();
        println!("Quote: {:?}", quote);
        assert!(quote.out_amount.parse::<u64>().unwrap() > 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_sol_price() {
        let client = JupiterClient::new("https://quote-api.jup.ag/v6".to_string());

        let price = client.get_sol_price_usd().await;
        assert!(price.is_ok());

        let price = price.unwrap();
        println!("SOL price: ${}", price);
        assert!(price > Decimal::ZERO);
    }
}
