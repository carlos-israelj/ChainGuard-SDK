use candid::{Nat, Principal};
use serde_bytes::ByteBuf;
use ethers_core::types::{
    transaction::eip1559::Eip1559TransactionRequest, Signature, U256, U64,
};
use ethers_core::utils::keccak256;
use evm_rpc_canister_types::{
    BlockTag, EthSepoliaService, FeeHistoryArgs, GetTransactionCountArgs, MultiGetTransactionCountResult,
    MultiFeeHistoryResult, MultiSendRawTransactionResult, RpcApi, RpcServices, SendRawTransactionStatus,
};
use ic_cdk::api::call::call_with_payment128;
use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, sign_with_ecdsa, EcdsaKeyId, EcdsaPublicKeyArgument, SignWithEcdsaArgument,
};
use num_bigint::BigUint;

/// EVM RPC Canister ID on IC mainnet
const EVM_RPC_CANISTER_ID: &str = "7hfb6-caaaa-aaaar-qadga-cai";
const CYCLES_PER_CALL: u128 = 10_000_000_000; // 10 billion cycles
const EIP1559_TX_ID: u8 = 2;

/// Signed transaction ready to send
#[derive(Debug, Clone)]
pub struct SignedTransaction {
    pub tx_hex: String,   // "0x02f86e..." format
    pub tx_hash: String,  // Transaction hash
}

/// Fee estimates from eth_feeHistory
#[derive(Debug, Clone)]
pub struct FeeEstimates {
    pub max_fee_per_gas: U256,
    pub max_priority_fee_per_gas: U256,
}

/// EVM RPC Executor using manual inter-canister calls
pub struct EvmRpcExecutor {
    evm_rpc_canister: Principal,
    key_name: String,
    derivation_path: Vec<Vec<u8>>,
}

impl EvmRpcExecutor {
    pub fn new(key_name: String, derivation_path: Vec<Vec<u8>>) -> Result<Self, String> {
        let principal = Principal::from_text(EVM_RPC_CANISTER_ID)
            .map_err(|e| format!("Invalid EVM RPC canister ID: {}", e))?;

        Ok(Self {
            evm_rpc_canister: principal,
            key_name,
            derivation_path,
        })
    }

    /// Execute a token transfer on the specified chain
    pub async fn transfer(
        &self,
        chain: &str,
        to: &str,
        amount: u64,
    ) -> Result<String, String> {
        // Get nonce for the sender address
        let from = self.get_eth_address().await?;
        let nonce = self.get_transaction_count(&from, chain).await?;

        // Estimate transaction fees
        let fee_estimates = self.estimate_transaction_fees(chain).await?;

        // Parse recipient address
        let to_addr: ethers_core::types::Address = to
            .parse()
            .map_err(|e| format!("Invalid recipient address: {:?}", e))?;

        // Build EIP-1559 transaction
        let tx = Eip1559TransactionRequest {
            from: None,
            to: Some(to_addr.into()),
            value: Some(U256::from(amount)),
            max_fee_per_gas: Some(fee_estimates.max_fee_per_gas),
            max_priority_fee_per_gas: Some(fee_estimates.max_priority_fee_per_gas),
            gas: Some(U256::from(30000)), // ETH transfer with buffer for testnet
            nonce: Some(nonce),
            chain_id: Some(self.get_chain_id(chain)?),
            data: Default::default(),
            access_list: Default::default(),
        };

        // Sign the transaction
        let signed_tx = self.sign_eip1559_transaction(tx).await?;

        // Send via EVM RPC canister
        self.send_raw_transaction(&signed_tx.tx_hex, chain).await?;

        Ok(signed_tx.tx_hash)
    }

    /// Sign an EIP-1559 transaction with Chain-Key ECDSA
    async fn sign_eip1559_transaction(
        &self,
        tx: Eip1559TransactionRequest,
    ) -> Result<SignedTransaction, String> {
        // Get the public key for this canister's derivation path
        let ecdsa_pub_key = self.get_canister_public_key().await?;

        // Create unsigned transaction bytes with EIP-1559 type prefix
        let mut unsigned_tx_bytes = tx.rlp().to_vec();
        unsigned_tx_bytes.insert(0, EIP1559_TX_ID);

        // Hash the unsigned transaction
        let txhash = keccak256(&unsigned_tx_bytes);

        // Sign with threshold ECDSA
        let key_id = EcdsaKeyId {
            curve: ic_cdk::api::management_canister::ecdsa::EcdsaCurve::Secp256k1,
            name: self.key_name.clone(),
        };

        let signature_result = sign_with_ecdsa(SignWithEcdsaArgument {
            message_hash: txhash.to_vec(),
            derivation_path: self.derivation_path.clone(),
            key_id,
        })
        .await
        .map_err(|e| format!("Failed to sign transaction: {:?}", e))?;

        let signature_bytes = signature_result.0.signature;

        // Construct Ethereum signature with v, r, s
        let v = self.y_parity(&txhash, &signature_bytes, &ecdsa_pub_key);
        let r = U256::from_big_endian(&signature_bytes[0..32]);
        let s = U256::from_big_endian(&signature_bytes[32..64]);

        let signature = Signature { v, r, s };

        // Create final signed transaction bytes
        let mut signed_tx_bytes = tx.rlp_signed(&signature).to_vec();
        signed_tx_bytes.insert(0, EIP1559_TX_ID);

        Ok(SignedTransaction {
            tx_hex: format!("0x{}", hex::encode(&signed_tx_bytes)),
            tx_hash: format!("0x{}", hex::encode(keccak256(&signed_tx_bytes))),
        })
    }

    /// Get the Ethereum address for this canister
    pub async fn get_eth_address(&self) -> Result<String, String> {
        let pubkey_bytes = self.get_canister_public_key().await?;
        Ok(self.pubkey_bytes_to_address(&pubkey_bytes))
    }

    /// Get the canister's ECDSA public key
    async fn get_canister_public_key(&self) -> Result<Vec<u8>, String> {
        let key_id = EcdsaKeyId {
            curve: ic_cdk::api::management_canister::ecdsa::EcdsaCurve::Secp256k1,
            name: self.key_name.clone(),
        };

        let (key,) = ecdsa_public_key(EcdsaPublicKeyArgument {
            canister_id: None,
            derivation_path: self.derivation_path.clone(),
            key_id,
        })
        .await
        .map_err(|e| format!("Failed to get public key: {:?}", e))?;

        Ok(key.public_key)
    }

    /// Convert public key bytes to Ethereum address
    fn pubkey_bytes_to_address(&self, pubkey_bytes: &[u8]) -> String {
        use ethers_core::k256::elliptic_curve::sec1::ToEncodedPoint;
        use ethers_core::k256::PublicKey;

        let key = PublicKey::from_sec1_bytes(pubkey_bytes)
            .expect("failed to parse the public key as SEC1");
        let point = key.to_encoded_point(false);
        let point_bytes = point.as_bytes();
        assert_eq!(point_bytes[0], 0x04);

        let hash = keccak256(&point_bytes[1..]);
        format!("0x{}", hex::encode(&hash[12..32]))
    }

    /// Calculate y_parity (v value) for ECDSA signature
    fn y_parity(&self, prehash: &[u8; 32], sig: &[u8], pubkey: &[u8]) -> u64 {
        use ethers_core::k256::ecdsa::{RecoveryId, Signature as K256Signature, VerifyingKey};

        let orig_key = VerifyingKey::from_sec1_bytes(pubkey).expect("failed to parse the pubkey");
        let signature = K256Signature::try_from(sig).unwrap();

        for parity in [0u8, 1] {
            let recid = RecoveryId::try_from(parity).unwrap();
            let recovered_key = VerifyingKey::recover_from_prehash(prehash, &signature, recid)
                .expect("failed to recover key");
            if recovered_key == orig_key {
                return parity as u64;
            }
        }

        panic!(
            "failed to recover the parity bit from a signature; sig: {}, pubkey: {}",
            hex::encode(sig),
            hex::encode(pubkey)
        )
    }

    /// Get the transaction count (nonce) for an address
    async fn get_transaction_count(&self, address: &str, chain: &str) -> Result<U256, String> {
        let rpc_service = self.get_rpc_service(chain)?;

        let args = GetTransactionCountArgs {
            address: address.to_string(),
            block: BlockTag::Latest,
        };

        let result: (MultiGetTransactionCountResult,) = call_with_payment128(
            self.evm_rpc_canister,
            "eth_getTransactionCount",
            (rpc_service, None::<()>, args),
            CYCLES_PER_CALL,
        )
        .await
        .map_err(|e| format!("Failed to call eth_getTransactionCount: {:?}", e))?;

        match result.0 {
            MultiGetTransactionCountResult::Consistent(count_result) => {
                match count_result {
                    evm_rpc_canister_types::GetTransactionCountResult::Ok(count) => {
                        self.nat_to_u256(&count)
                    }
                    evm_rpc_canister_types::GetTransactionCountResult::Err(e) => {
                        Err(format!("RPC error: {:?}", e))
                    }
                }
            }
            MultiGetTransactionCountResult::Inconsistent(_) => {
                Err("Inconsistent results from RPC providers".to_string())
            }
        }
    }

    /// Estimate transaction fees using eth_feeHistory
    async fn estimate_transaction_fees(&self, chain: &str) -> Result<FeeEstimates, String> {
        let rpc_service = self.get_rpc_service(chain)?;

        let args = FeeHistoryArgs {
            blockCount: Nat::from(9u8),
            newestBlock: BlockTag::Latest,
            rewardPercentiles: Some(ByteBuf::from(vec![95u8])),
        };

        let result: (MultiFeeHistoryResult,) = call_with_payment128(
            self.evm_rpc_canister,
            "eth_feeHistory",
            (rpc_service, None::<()>, args),
            CYCLES_PER_CALL,
        )
        .await
        .map_err(|e| format!("Failed to call eth_feeHistory: {:?}", e))?;

        let fee_history = match result.0 {
            MultiFeeHistoryResult::Consistent(history_result) => {
                match history_result {
                    evm_rpc_canister_types::FeeHistoryResult::Ok(history) => history,
                    evm_rpc_canister_types::FeeHistoryResult::Err(e) => {
                        return Err(format!("RPC error: {:?}", e));
                    }
                }
            }
            MultiFeeHistoryResult::Inconsistent(_) => {
                return Err("Inconsistent results from RPC providers".to_string());
            }
        };

        let base_fee_per_gas = fee_history
            .baseFeePerGas
            .last()
            .ok_or("No base fee available")?;

        let rewards = fee_history.reward;
        let mut percentile_95: Vec<Nat> = rewards
            .into_iter()
            .flat_map(|x| x.into_iter())
            .collect();
        percentile_95.sort();

        let median_reward = percentile_95
            .get(percentile_95.len() / 2)
            .unwrap_or(&Nat::from(0u8))
            .clone();

        let max_priority_fee_per_gas = self.nat_to_u256(&median_reward)?;
        let max_fee_per_gas = self.nat_to_u256(base_fee_per_gas)? + max_priority_fee_per_gas;

        Ok(FeeEstimates {
            max_fee_per_gas,
            max_priority_fee_per_gas,
        })
    }

    /// Send a raw signed transaction
    async fn send_raw_transaction(&self, raw_tx: &str, chain: &str) -> Result<(), String> {
        let rpc_service = self.get_rpc_service(chain)?;

        let result: (MultiSendRawTransactionResult,) = call_with_payment128(
            self.evm_rpc_canister,
            "eth_sendRawTransaction",
            (rpc_service, None::<()>, raw_tx.to_string()),
            CYCLES_PER_CALL,
        )
        .await
        .map_err(|e| format!("Failed to call eth_sendRawTransaction: {:?}", e))?;

        match result.0 {
            MultiSendRawTransactionResult::Consistent(send_result) => {
                match send_result {
                    evm_rpc_canister_types::SendRawTransactionResult::Ok(status) => {
                        match status {
                            SendRawTransactionStatus::Ok(Some(_)) => Ok(()),
                            SendRawTransactionStatus::Ok(None) => Err("No transaction hash returned".to_string()),
                            SendRawTransactionStatus::NonceTooLow => Err("Nonce too low".to_string()),
                            SendRawTransactionStatus::NonceTooHigh => Err("Nonce too high".to_string()),
                            SendRawTransactionStatus::InsufficientFunds => Err("Insufficient funds".to_string()),
                        }
                    }
                    evm_rpc_canister_types::SendRawTransactionResult::Err(e) => {
                        Err(format!("RPC error: {:?}", e))
                    }
                }
            }
            MultiSendRawTransactionResult::Inconsistent(_) => {
                Err("Inconsistent results from RPC providers".to_string())
            }
        }
    }

    /// Get RPC service for a chain
    fn get_rpc_service(&self, chain: &str) -> Result<RpcServices, String> {
        match chain.to_lowercase().as_str() {
            "sepolia" => {
                // Use custom RPC with Alchemy API key for better consistency
                Ok(RpcServices::Custom {
                    chainId: 11155111, // Sepolia chain ID
                    services: vec![RpcApi {
                        url: crate::config::get_alchemy_sepolia_url(),
                        headers: None,
                    }],
                })
            }
            _ => Err(format!("Unsupported chain: {} (only Sepolia for now)", chain)),
        }
    }

    /// Get chain ID for a chain
    fn get_chain_id(&self, chain: &str) -> Result<U64, String> {
        match chain.to_lowercase().as_str() {
            "sepolia" => Ok(U64::from(11155111)),
            _ => Err(format!("Unknown chain ID for: {}", chain)),
        }
    }

    /// Convert Candid Nat to U256
    fn nat_to_u256(&self, n: &Nat) -> Result<U256, String> {
        let big_uint: BigUint = n.0.clone();
        let bytes = big_uint.to_bytes_be();

        if bytes.len() > 32 {
            return Err("Number too large for U256".to_string());
        }

        Ok(U256::from_big_endian(&bytes))
    }
}
