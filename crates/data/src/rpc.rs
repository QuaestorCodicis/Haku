use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::RpcTransactionConfig,
    rpc_response::RpcConfirmedTransactionStatusWithSignature,
};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::Signature,
};
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use trading_core::{Result, TradingError};

/// RPC client with automatic fallback to backup endpoints
pub struct FallbackRpcClient {
    primary: Arc<RpcClient>,
    fallbacks: Vec<Arc<RpcClient>>,
    current_index: Arc<RwLock<usize>>,
    commitment: CommitmentConfig,
}

impl FallbackRpcClient {
    pub fn new(primary_url: String, fallback_urls: Vec<String>, commitment: CommitmentConfig) -> Self {
        let primary = Arc::new(RpcClient::new_with_timeout_and_commitment(
            primary_url.clone(),
            Duration::from_secs(30),
            commitment,
        ));

        let fallbacks: Vec<Arc<RpcClient>> = fallback_urls
            .into_iter()
            .map(|url| {
                Arc::new(RpcClient::new_with_timeout_and_commitment(
                    url,
                    Duration::from_secs(30),
                    commitment,
                ))
            })
            .collect();

        info!("Initialized RPC client with {} fallback endpoints", fallbacks.len());

        Self {
            primary,
            fallbacks,
            current_index: Arc::new(RwLock::new(0)),
            commitment,
        }
    }

    /// Get current active RPC client
    async fn get_client(&self) -> Arc<RpcClient> {
        let index = *self.current_index.read().await;
        if index == 0 {
            self.primary.clone()
        } else {
            self.fallbacks
                .get(index - 1)
                .cloned()
                .unwrap_or_else(|| self.primary.clone())
        }
    }

    /// Switch to next fallback endpoint
    async fn switch_to_fallback(&self) {
        let mut index = self.current_index.write().await;
        let max_index = self.fallbacks.len();
        *index = (*index + 1) % (max_index + 1);

        if *index == 0 {
            warn!("Switched back to primary RPC endpoint");
        } else {
            warn!("Switched to fallback RPC endpoint #{}", *index);
        }
    }

    /// Execute RPC call with automatic fallback
    async fn execute_with_fallback<F, T, Fut>(&self, operation: F) -> Result<T>
    where
        F: Fn(Arc<RpcClient>) -> Fut,
        Fut: std::future::Future<Output = std::result::Result<T, solana_client::client_error::ClientError>>,
    {
        let mut attempts = 0;
        let max_attempts = self.fallbacks.len() + 1;

        loop {
            let client = self.get_client().await;

            match operation(client).await {
                Ok(result) => {
                    if attempts > 0 {
                        debug!("RPC call succeeded after {} attempts", attempts + 1);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    attempts += 1;
                    error!("RPC call failed (attempt {}/{}): {}", attempts, max_attempts, e);

                    if attempts >= max_attempts {
                        return Err(TradingError::RpcError(format!(
                            "All RPC endpoints failed after {} attempts",
                            attempts
                        )));
                    }

                    self.switch_to_fallback().await;
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
        }
    }

    /// Get account balance
    pub async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64> {
        self.execute_with_fallback(|client| async move {
            client.get_balance(pubkey).await
        })
        .await
    }

    /// Get transaction
    pub async fn get_transaction(
        &self,
        signature: &Signature,
    ) -> Result<EncodedConfirmedTransactionWithStatusMeta> {
        self.execute_with_fallback(|client| async move {
            client
                .get_transaction_with_config(
                    signature,
                    RpcTransactionConfig {
                        encoding: Some(UiTransactionEncoding::JsonParsed),
                        commitment: Some(self.commitment),
                        max_supported_transaction_version: Some(0),
                    },
                )
                .await
        })
        .await
    }

    /// Get signatures for address
    pub async fn get_signatures_for_address(
        &self,
        address: &Pubkey,
        limit: usize,
    ) -> Result<Vec<RpcConfirmedTransactionStatusWithSignature>> {
        self.execute_with_fallback(|client| async move {
            client
                .get_signatures_for_address_with_config(
                    address,
                    solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config {
                        limit: Some(limit),
                        ..Default::default()
                    },
                )
                .await
        })
        .await
    }

    /// Get token account balance
    pub async fn get_token_account_balance(&self, pubkey: &Pubkey) -> Result<u64> {
        self.execute_with_fallback(|client| async move {
            client
                .get_token_account_balance(pubkey)
                .await
                .map(|balance| balance.amount.parse::<u64>().unwrap_or(0))
        })
        .await
    }

    /// Get slot
    pub async fn get_slot(&self) -> Result<u64> {
        self.execute_with_fallback(|client| async move { client.get_slot().await })
            .await
    }

    /// Get block time
    pub async fn get_block_time(&self, slot: u64) -> Result<i64> {
        self.execute_with_fallback(|client| async move {
            client.get_block_time(slot).await
        })
        .await
    }

    /// Get latest blockhash
    pub async fn get_latest_blockhash(&self) -> Result<solana_sdk::hash::Hash> {
        self.execute_with_fallback(|client| async move {
            client.get_latest_blockhash().await
        })
        .await
    }

    /// Send transaction
    pub async fn send_transaction(
        &self,
        transaction: &solana_sdk::transaction::Transaction,
    ) -> Result<Signature> {
        self.execute_with_fallback(|client| async move {
            client.send_transaction(transaction).await
        })
        .await
    }

    /// Get account data
    pub async fn get_account_data(&self, pubkey: &Pubkey) -> Result<Vec<u8>> {
        self.execute_with_fallback(|client| async move {
            client.get_account_data(pubkey).await
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rpc_fallback() {
        let client = FallbackRpcClient::new(
            "https://api.mainnet-beta.solana.com".to_string(),
            vec![
                "https://solana-api.projectserum.com".to_string(),
                "https://rpc.ankr.com/solana".to_string(),
            ],
            CommitmentConfig::confirmed(),
        );

        // Test getting slot
        let slot = client.get_slot().await;
        assert!(slot.is_ok());
    }
}
