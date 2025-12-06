use crate::types::*;
use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, sign_with_ecdsa, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument,
    SignWithEcdsaArgument,
};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::icp::IcpSigner;
use alloy::transports::icp::{IcpConfig, RpcService, EthMainnetService, EthSepoliaService, L2MainnetService};
use alloy::primitives::{Address, U256};
use alloy::rpc::types::TransactionRequest;

/// Multi-chain transaction executor using Chain-Key ECDSA and ic-alloy
#[derive(Clone)]
pub struct ChainExecutor {
    key_name: String,
    derivation_path: Vec<Vec<u8>>,
}

impl ChainExecutor {
    pub fn new(key_name: String) -> Self {
        // Use canister ID as derivation path for unique keys per canister
        let derivation_path = vec![ic_cdk::id().as_slice().to_vec()];

        Self {
            key_name,
            derivation_path,
        }
    }

    /// Get the Ethereum address for this canister's ECDSA key
    pub async fn get_eth_address(&self) -> Result<String, String> {
        let key_id = EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: self.key_name.clone(),
        };

        let arg = EcdsaPublicKeyArgument {
            canister_id: None,
            derivation_path: self.derivation_path.clone(),
            key_id: key_id.clone(),
        };

        // Get public key from management canister
        let (response,) = ecdsa_public_key(arg)
            .await
            .map_err(|e| format!("Failed to get public key: {:?}", e))?;

        // Convert public key to Ethereum address
        // Public key is 33 bytes (compressed), need to derive address
        let pubkey = response.public_key;

        // For production: properly derive Ethereum address from secp256k1 public key
        // For now, return a placeholder that indicates successful key retrieval
        Ok(format!("0x{}", hex::encode(&pubkey[..20])))
    }

    /// Execute an action on the specified chain
    pub async fn execute_action(&self, action: &Action) -> ExecutionResult {
        match action {
            Action::Transfer { chain, token, to, amount } => {
                self.execute_transfer(chain, token, to, *amount).await
            }
            Action::Swap { chain, token_in, token_out, amount_in, min_amount_out } => {
                self.execute_swap(chain, token_in, token_out, *amount_in, *min_amount_out).await
            }
            Action::ApproveToken { chain, token, spender, amount } => {
                self.execute_approve(chain, token, spender, *amount).await
            }
        }
    }

    /// Execute a token transfer
    async fn execute_transfer(
        &self,
        chain: &str,
        _token: &str,
        to: &str,
        amount: u64,
    ) -> ExecutionResult {
        // Get RPC service for the chain
        let rpc_service = match self.get_rpc_service(chain) {
            Ok(service) => service,
            Err(e) => {
                return ExecutionResult {
                    success: false,
                    chain: chain.to_string(),
                    tx_hash: None,
                    error: Some(e),
                }
            }
        };

        // Create ICP config
        let config = IcpConfig::new(rpc_service);

        // Build provider with ICP transport
        let provider = ProviderBuilder::new().on_icp(config);

        // Create ICP signer
        let signer = IcpSigner::new(
            self.derivation_path.clone(),
            &self.key_name,
            None,
        );

        // Parse recipient address
        let to_addr: Address = match to.parse() {
            Ok(addr) => addr,
            Err(e) => {
                return ExecutionResult {
                    success: false,
                    chain: chain.to_string(),
                    tx_hash: None,
                    error: Some(format!("Invalid address: {:?}", e)),
                }
            }
        };

        // Build transaction
        let tx = TransactionRequest::default()
            .to(to_addr)
            .value(U256::from(amount));

        // Send transaction
        match provider.send_transaction(tx).await {
            Ok(pending) => {
                let tx_hash = format!("{:?}", pending.tx_hash());
                ExecutionResult {
                    success: true,
                    chain: chain.to_string(),
                    tx_hash: Some(tx_hash),
                    error: None,
                }
            }
            Err(e) => ExecutionResult {
                success: false,
                chain: chain.to_string(),
                tx_hash: None,
                error: Some(format!("Transaction failed: {:?}", e)),
            },
        }
    }

    /// Execute a token swap (placeholder - requires DEX integration)
    async fn execute_swap(
        &self,
        chain: &str,
        _token_in: &str,
        _token_out: &str,
        _amount_in: u64,
        _min_amount_out: u64,
    ) -> ExecutionResult {
        // TODO: Implement DEX integration (Uniswap, etc.)
        ExecutionResult {
            success: false,
            chain: chain.to_string(),
            tx_hash: None,
            error: Some("Swap not yet implemented - requires DEX integration".to_string()),
        }
    }

    /// Execute a token approval
    async fn execute_approve(
        &self,
        chain: &str,
        _token: &str,
        _spender: &str,
        _amount: u64,
    ) -> ExecutionResult {
        // TODO: Implement ERC20 approve call
        ExecutionResult {
            success: false,
            chain: chain.to_string(),
            tx_hash: None,
            error: Some("Approve not yet implemented - requires ERC20 ABI".to_string()),
        }
    }

    /// Get RPC service for a specific chain
    fn get_rpc_service(&self, chain: &str) -> Result<RpcService, String> {
        use alloy::transports::icp::{EthMainnetService, EthSepoliaService, L2MainnetService};

        match chain.to_lowercase().as_str() {
            "ethereum" | "eth" => Ok(RpcService::EthMainnet(EthMainnetService::Cloudflare)),
            "sepolia" => Ok(RpcService::EthSepolia(EthSepoliaService::Ankr)),
            "arbitrum" | "arbitrum-one" => Ok(RpcService::ArbitrumOne(L2MainnetService::Ankr)),
            "optimism" | "op" => Ok(RpcService::OptimismMainnet(L2MainnetService::Ankr)),
            "base" => Ok(RpcService::BaseMainnet(L2MainnetService::Ankr)),
            _ => Err(format!("Unsupported chain: {}. Supported: ethereum, sepolia, arbitrum, optimism, base", chain)),
        }
    }

    /// Sign a message with Chain-Key ECDSA
    pub async fn sign_message(&self, message: &[u8]) -> Result<Vec<u8>, String> {
        let key_id = EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: self.key_name.clone(),
        };

        let arg = SignWithEcdsaArgument {
            message_hash: message.to_vec(),
            derivation_path: self.derivation_path.clone(),
            key_id,
        };

        let (response,) = sign_with_ecdsa(arg)
            .await
            .map_err(|e| format!("Failed to sign: {:?}", e))?;

        Ok(response.signature)
    }
}

impl Default for ChainExecutor {
    fn default() -> Self {
        // Use test key for local development, production key for mainnet
        Self::new("dfx_test_key".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_rpc_service() {
        let executor = ChainExecutor::default();

        assert!(executor.get_rpc_service("ethereum").is_ok());
        assert!(executor.get_rpc_service("sepolia").is_ok());
        assert!(executor.get_rpc_service("arbitrum").is_ok());
        assert!(executor.get_rpc_service("optimism").is_ok());
        assert!(executor.get_rpc_service("base").is_ok());

        assert!(executor.get_rpc_service("unsupported").is_err());
    }

    #[test]
    fn test_executor_creation() {
        let executor = ChainExecutor::new("test_key".to_string());
        assert_eq!(executor.key_name, "test_key");
        assert_eq!(executor.derivation_path.len(), 1);
    }
}
