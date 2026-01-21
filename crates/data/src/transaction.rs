use chrono::{DateTime, Utc};
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, UiInstruction, UiMessage, UiParsedInstruction,
    UiTransaction, UiTransactionTokenBalance,
};
use std::str::FromStr;
use tracing::{debug, warn};
use trading_core::{Result, Trade, TradeSide, TradingError};
use rust_decimal::Decimal;
use uuid::Uuid;

/// Transaction parser for extracting trade data
pub struct TransactionParser;

impl TransactionParser {
    /// Parse a transaction and extract trade information
    pub fn parse_trade(
        transaction: &EncodedConfirmedTransactionWithStatusMeta,
        wallet: &Pubkey,
    ) -> Result<Option<Trade>> {
        // EncodedConfirmedTransactionWithStatusMeta contains transaction and meta fields
        let encoded_tx_with_meta = &transaction.transaction;

        let tx = match &encoded_tx_with_meta.transaction {
            solana_transaction_status::EncodedTransaction::Json(tx) => tx,
            _ => return Ok(None),
        };

        // Get metadata from the EncodedTransactionWithStatusMeta
        let meta_data = encoded_tx_with_meta.meta.as_ref()
            .ok_or_else(|| TradingError::ParseError("Transaction missing metadata".to_string()))?;

        // Check if transaction was successful
        if meta_data.err.is_some() {
            return Ok(None);
        }

        // Extract pre and post token balances
        let empty_vec = vec![];
        let pre_balances = match &meta_data.pre_token_balances {
            solana_transaction_status::option_serializer::OptionSerializer::Some(v) => v.as_slice(),
            _ => &empty_vec,
        };
        let post_balances = match &meta_data.post_token_balances {
            solana_transaction_status::option_serializer::OptionSerializer::Some(v) => v.as_slice(),
            _ => &empty_vec,
        };

        // Extract signature from transaction
        let signature = tx.signatures.first()
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        // Detect token swap by analyzing balance changes
        if let Some(trade) = Self::detect_swap(wallet, pre_balances, post_balances, &tx.message, transaction.block_time, signature)? {
            return Ok(Some(trade));
        }

        Ok(None)
    }

    /// Detect swap transaction
    fn detect_swap(
        wallet: &Pubkey,
        pre_balances: &[UiTransactionTokenBalance],
        post_balances: &[UiTransactionTokenBalance],
        message: &UiMessage,
        block_time: Option<i64>,
        signature: String,
    ) -> Result<Option<Trade>> {
        // Find balance changes for this wallet
        let mut balance_changes: Vec<(String, i128)> = Vec::new();

        for post in post_balances {
            let owner_str = match &post.owner {
                solana_transaction_status::option_serializer::OptionSerializer::Some(s) => s,
                _ => continue,
            };
            let owner = Self::extract_owner(owner_str)?;
            if owner != *wallet {
                continue;
            }

            let mint = post.mint.clone();
            let post_amount = Self::parse_token_amount(&post.ui_token_amount.amount)?;

            // Find corresponding pre balance
            let pre_amount = pre_balances
                .iter()
                .find(|pre| {
                    if let solana_transaction_status::option_serializer::OptionSerializer::Some(pre_owner) = &pre.owner {
                        pre.mint == mint && Self::extract_owner(pre_owner).ok() == Some(owner)
                    } else {
                        false
                    }
                })
                .and_then(|pre| Self::parse_token_amount(&pre.ui_token_amount.amount).ok())
                .unwrap_or(0);

            let change = post_amount - pre_amount;
            if change != 0 {
                balance_changes.push((mint, change));
            }
        }

        // Swap should have exactly 2 balance changes (one positive, one negative)
        if balance_changes.len() != 2 {
            return Ok(None);
        }

        // Determine which is input and which is output
        let (token_in, amount_in_raw) = balance_changes
            .iter()
            .find(|(_, change)| *change < 0)
            .ok_or_else(|| TradingError::ParseError("No negative balance change found".to_string()))?;
        let (token_out, amount_out_raw) = balance_changes
            .iter()
            .find(|(_, change)| *change > 0)
            .ok_or_else(|| TradingError::ParseError("No positive balance change found".to_string()))?;

        let token_mint_in = Pubkey::from_str(token_in)
            .map_err(|e| TradingError::ParseError(format!("Invalid mint: {}", e)))?;
        let token_mint_out = Pubkey::from_str(token_out)
            .map_err(|e| TradingError::ParseError(format!("Invalid mint: {}", e)))?;

        let amount_in = Decimal::from_i128_with_scale(amount_in_raw.abs(), 9);
        let amount_out = Decimal::from_i128_with_scale(*amount_out_raw, 9);

        // Determine if this is a buy or sell (relative to SOL or USDC)
        let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
        let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();

        let (side, token_mint) = if token_mint_in == sol_mint || token_mint_in == usdc_mint {
            (TradeSide::Buy, token_mint_out)
        } else {
            (TradeSide::Sell, token_mint_in)
        };

        // Extract DEX name from instructions
        let dex = Self::extract_dex_name(message).unwrap_or_else(|| "Unknown".to_string());

        let trade = Trade {
            id: Uuid::new_v4(),
            wallet: *wallet,
            token_mint,
            side,
            amount_in,
            amount_out,
            price_usd: Decimal::ZERO, // Will be enriched later
            market_cap_at_trade: Decimal::ZERO, // Will be enriched later
            signature,
            timestamp: block_time
                .map(|t| DateTime::from_timestamp(t, 0).unwrap_or_else(|| Utc::now()))
                .unwrap_or_else(|| Utc::now()),
            block_time: block_time.unwrap_or(0),
            dex: dex.clone(),
        };

        debug!("Detected {:?} trade for {} on {}", side, token_mint, dex);

        Ok(Some(trade))
    }

    /// Extract owner pubkey from owner string
    fn extract_owner(owner: &str) -> Result<Pubkey> {
        Pubkey::from_str(owner)
            .map_err(|e| TradingError::ParseError(format!("Invalid owner pubkey: {}", e)))
    }

    /// Parse token amount string to i128
    fn parse_token_amount(amount: &str) -> Result<i128> {
        amount
            .parse::<i128>()
            .map_err(|e| TradingError::ParseError(format!("Invalid amount: {}", e)))
    }

    /// Extract DEX name from transaction instructions
    fn extract_dex_name(message: &UiMessage) -> Option<String> {
        let instructions = match message {
            UiMessage::Parsed(parsed) => &parsed.instructions,
            _ => return None,
        };

        for instruction in instructions {
            if let UiInstruction::Parsed(UiParsedInstruction::PartiallyDecoded(decoded)) = instruction {
                let program = &decoded.program_id;

                // Known DEX program IDs
                if program.contains("JUP") || program.contains("Jupiter") {
                    return Some("Jupiter".to_string());
                } else if program.contains("RVKd61ztZW9GUwhRbbLoYVRE5Xf1B2tVscKqwZqXgEr") {
                    return Some("Raydium".to_string());
                } else if program.contains("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc") {
                    return Some("Orca".to_string());
                } else if program.contains("Pump") || program.contains("pump") {
                    return Some("Pump.fun".to_string());
                }
            }
        }

        None
    }

    /// Get all trades for a wallet from transaction history
    pub async fn get_wallet_trades(
        rpc_client: &crate::rpc::FallbackRpcClient,
        wallet: &Pubkey,
        limit: usize,
    ) -> Result<Vec<Trade>> {
        let signatures = rpc_client.get_signatures_for_address(wallet, limit).await?;
        let sig_count = signatures.len();

        let mut trades = Vec::new();

        for sig_info in signatures {
            let signature = solana_sdk::signature::Signature::from_str(&sig_info.signature)
                .map_err(|e| TradingError::ParseError(format!("Invalid signature: {}", e)))?;

            match rpc_client.get_transaction(&signature).await {
                Ok(transaction) => {
                    if let Some(trade) = Self::parse_trade(&transaction, wallet)? {
                        trades.push(trade);
                    }
                }
                Err(e) => {
                    warn!("Failed to fetch transaction {}: {}", sig_info.signature, e);
                    continue;
                }
            }

            // Rate limiting
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        debug!("Extracted {} trades from {} transactions", trades.len(), sig_count);

        Ok(trades)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_dex_name() {
        // Test would require actual transaction data
        // Placeholder for now
    }
}
