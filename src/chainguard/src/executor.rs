use crate::types::*;
use crate::evm_rpc::EvmRpcExecutor;
use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, sign_with_ecdsa, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument, SignWithEcdsaArgument,
};

/// Multi-chain transaction executor using Chain-Key ECDSA and ic-alloy
#[derive(Clone)]
pub struct ChainExecutor {
    pub key_name: String,
    pub derivation_path: Vec<Vec<u8>>,
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
            Action::Swap { chain, token_in, token_out, amount_in, min_amount_out, fee_tier } => {
                self.execute_swap(chain, token_in, token_out, *amount_in, *min_amount_out, *fee_tier).await
            }
            Action::ApproveToken { chain, token, spender, amount } => {
                self.execute_approve(chain, token, spender, *amount).await
            }
        }
    }

    /// Execute a token transfer using EVM RPC canister
    async fn execute_transfer(
        &self,
        chain: &str,
        _token: &str,
        to: &str,
        amount: u64,
    ) -> ExecutionResult {
        // Create EVM RPC executor
        let evm_executor = match EvmRpcExecutor::new(
            self.key_name.clone(),
            self.derivation_path.clone(),
        ) {
            Ok(executor) => executor,
            Err(e) => {
                return ExecutionResult {
                    success: false,
                    chain: chain.to_string(),
                    tx_hash: None,
                    error: Some(format!("Failed to create EVM RPC executor: {}", e)),
                }
            }
        };

        // Execute the transfer via EVM RPC canister
        match evm_executor.transfer(chain, to, amount).await {
            Ok(tx_hash) => ExecutionResult {
                success: true,
                chain: chain.to_string(),
                tx_hash: Some(tx_hash),
                error: None,
            },
            Err(e) => ExecutionResult {
                success: false,
                chain: chain.to_string(),
                tx_hash: None,
                error: Some(format!("Transaction failed: {}", e)),
            },
        }
    }

    /// Execute a token swap via Uniswap Universal Router
    async fn execute_swap(
        &self,
        chain: &str,
        token_in: &str,
        token_out: &str,
        amount_in: u64,
        min_amount_out: u64,
        fee_tier: Option<u32>,
    ) -> ExecutionResult {
        use crate::universal_router::{self, commands, special_addresses};
        use crate::abi::erc20;
        use ethers_core::types::{Address, U256};
        use ic_cdk::api::time;

        // Get Universal Router address for the chain
        let router_address = match universal_router::get_universal_router_address(chain) {
            Some(addr) => addr,
            None => return ExecutionResult {
                success: false,
                chain: chain.to_string(),
                tx_hash: None,
                error: Some(format!("Universal Router not available for chain: {}", chain)),
            },
        };

        // WETH9 address for the chain (official Uniswap V3 WETH)
        let weth_address = match chain.to_lowercase().as_str() {
            "sepolia" => "0xfff9976782d46cc05630d1f6ebab18b2324d6b14", // Sepolia WETH9
            "ethereum" | "mainnet" => "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // Mainnet WETH9
            _ => return ExecutionResult {
                success: false,
                chain: chain.to_string(),
                tx_hash: None,
                error: Some(format!("WETH not configured for chain: {}", chain)),
            },
        };

        // Uniswap V3 Fee Tiers (basis points where 1 bp = 0.01%):
        // 100 (0.01%) - Stablecoins
        // 500 (0.05%) - Low volatility pairs
        // 3000 (0.30%) - Standard pairs (default)
        // 10000 (1.00%) - Exotic/rare pairs
        let fee_tier: u32 = fee_tier.unwrap_or(3000);
        ic_cdk::println!("🔧 Using fee tier: {} ({:.2}%)", fee_tier, fee_tier as f64 / 10000.0);

        // Create EVM RPC executor
        let evm_executor = match EvmRpcExecutor::new(
            self.key_name.clone(),
            self.derivation_path.clone(),
        ) {
            Ok(executor) => executor,
            Err(e) => {
                return ExecutionResult {
                    success: false,
                    chain: chain.to_string(),
                    tx_hash: None,
                    error: Some(format!("Failed to create EVM RPC executor: {}", e)),
                }
            }
        };

        // Check if this is an ETH swap
        let is_eth_in = token_in.to_uppercase() == "ETH";
        let is_eth_out = token_out.to_uppercase() == "ETH";

        // Parse WETH address
        let weth_addr: Address = match weth_address.parse() {
            Ok(addr) => addr,
            Err(e) => return ExecutionResult {
                success: false,
                chain: chain.to_string(),
                tx_hash: None,
                error: Some(format!("Invalid WETH address: {:?}", e)),
            },
        };

        // Determine actual token addresses for the swap
        let (actual_token_in, actual_token_out, needs_wrap, needs_unwrap) = if is_eth_in {
            // ETH -> Token: WETH is token_in
            let token_out_addr: Address = match token_out.parse() {
                Ok(addr) => addr,
                Err(e) => return ExecutionResult {
                    success: false,
                    chain: chain.to_string(),
                    tx_hash: None,
                    error: Some(format!("Invalid token_out address: {:?}", e)),
                },
            };
            (weth_addr, token_out_addr, true, false)
        } else if is_eth_out {
            // Token -> ETH: WETH is token_out
            let token_in_addr: Address = match token_in.parse() {
                Ok(addr) => addr,
                Err(e) => return ExecutionResult {
                    success: false,
                    chain: chain.to_string(),
                    tx_hash: None,
                    error: Some(format!("Invalid token_in address: {:?}", e)),
                },
            };
            (token_in_addr, weth_addr, false, true)
        } else {
            // Token -> Token
            let token_in_addr: Address = match token_in.parse() {
                Ok(addr) => addr,
                Err(e) => return ExecutionResult {
                    success: false,
                    chain: chain.to_string(),
                    tx_hash: None,
                    error: Some(format!("Invalid token_in address: {:?}", e)),
                },
            };
            let token_out_addr: Address = match token_out.parse() {
                Ok(addr) => addr,
                Err(e) => return ExecutionResult {
                    success: false,
                    chain: chain.to_string(),
                    tx_hash: None,
                    error: Some(format!("Invalid token_out address: {:?}", e)),
                },
            };
            (token_in_addr, token_out_addr, false, false)
        };

        // Get recipient address (the canister's own address)
        let recipient = match self.get_eth_address().await {
            Ok(addr) => match addr.parse::<Address>() {
                Ok(a) => a,
                Err(e) => return ExecutionResult {
                    success: false,
                    chain: chain.to_string(),
                    tx_hash: None,
                    error: Some(format!("Failed to parse ETH address: {:?}", e)),
                },
            },
            Err(e) => return ExecutionResult {
                success: false,
                chain: chain.to_string(),
                tx_hash: None,
                error: Some(format!("Failed to get ETH address: {}", e)),
            },
        };

        // Parse Universal Router address
        let router_addr: Address = match router_address.parse() {
            Ok(addr) => addr,
            Err(e) => return ExecutionResult {
                success: false,
                chain: chain.to_string(),
                tx_hash: None,
                error: Some(format!("Invalid router address: {:?}", e)),
            },
        };

        // Calculate deadline (current time + 15 minutes)
        let deadline = (time() / 1_000_000_000) + 900; // 15 minutes from now

        // Validate balance before attempting swap (simplified check)
        // Note: Balance validation will also happen during transaction execution
        if needs_wrap {
            // For ETH swaps, log that we're checking balance
            if let Err(e) = evm_executor.check_eth_balance(&recipient.to_string(), U256::from(amount_in)).await {
                return ExecutionResult {
                    success: false,
                    chain: chain.to_string(),
                    tx_hash: None,
                    error: Some(format!("Balance check failed: {}", e)),
                };
            }
        } else {
            // For token swaps, log that we're checking balance
            if let Err(e) = evm_executor.check_token_balance(
                token_in,
                &recipient.to_string(),
                U256::from(amount_in)
            ).await {
                return ExecutionResult {
                    success: false,
                    chain: chain.to_string(),
                    tx_hash: None,
                    error: Some(format!("Balance check failed: {}", e)),
                };
            }
        }

        // Build commands and inputs for Universal Router
        let mut cmd_list = Vec::new();
        let mut input_list = Vec::new();

        // Step 1: If ETH input, wrap it to WETH
        if needs_wrap {
            cmd_list.push(commands::WRAP_ETH);
            // WRAP_ETH expects: (recipient, amountMin)
            // recipient = ADDRESS_THIS (router holds it temporarily)
            let router_as_recipient: Address = special_addresses::ADDRESS_THIS.parse().unwrap();
            input_list.push(universal_router::encode_wrap_eth(router_as_recipient, U256::from(amount_in)));
        }

        // Step 2: If token input (not ETH), approve Permit2 and wait for confirmation
        // Universal Router uses Permit2 to pull tokens, not direct approval
        // We'll approve with a large amount to minimize future approvals
        if !needs_wrap {
            ic_cdk::println!("🔐 Token swap detected - approving Permit2...");

            // Permit2 address (same across all networks - deterministic deployment)
            let permit2_addr: Address = "0x000000000022D473030F116dDEE9F6B43aC78BA3".parse().unwrap();

            // Approve Permit2 to spend tokens (use 10x the swap amount for buffer)
            let approval_amount = U256::from(amount_in) * U256::from(10);
            let approve_call_data = erc20::encode_approve(permit2_addr, approval_amount);

            match evm_executor.call_contract(chain, token_in, approve_call_data, 0).await {
                Ok(tx_hash) => {
                    ic_cdk::println!("✅ Token approval to Permit2 sent: {}", tx_hash);

                    // CRITICAL: Wait for approval to be confirmed before proceeding
                    ic_cdk::println!("⏳ Waiting for token approval confirmation (max 10 attempts)...");
                    match evm_executor.wait_for_confirmation(&tx_hash, chain, 10).await {
                        Ok(_) => {
                            ic_cdk::println!("✅ Token approval confirmed!");
                        }
                        Err(e) => {
                            ic_cdk::println!("⚠️ Could not confirm token approval: {}", e);
                            ic_cdk::println!("⚠️ Continuing anyway - might have existing approval");
                        }
                    }
                }
                Err(e) => {
                    ic_cdk::println!("⚠️ Token approval transaction error: {}", e);
                    ic_cdk::println!("⚠️ Will attempt Permit2 approval anyway");
                }
            }

            // Step 2b: Approve Universal Router in Permit2 to spend tokens via AllowanceTransfer
            ic_cdk::println!("🔐 Approving Universal Router in Permit2...");

            // Calculate expiration (30 days from now)
            let expiration = (time() / 1_000_000_000) + (30 * 24 * 60 * 60); // 30 days

            // Encode Permit2.approve(token, spender, amount, expiration)
            let permit2_approve_data = crate::abi::permit2::encode_approve(
                actual_token_in,
                router_address.parse().unwrap(),
                approval_amount,
                expiration,
            );

            match evm_executor.call_contract(chain, "0x000000000022D473030F116dDEE9F6B43aC78BA3", permit2_approve_data, 0).await {
                Ok(tx_hash) => {
                    ic_cdk::println!("✅ Permit2 approval sent: {}", tx_hash);

                    // Wait for Permit2 approval to be confirmed
                    ic_cdk::println!("⏳ Waiting for Permit2 approval confirmation (max 10 attempts)...");
                    match evm_executor.wait_for_confirmation(&tx_hash, chain, 10).await {
                        Ok(_) => {
                            ic_cdk::println!("✅ Permit2 approval confirmed! Proceeding with swap...");
                        }
                        Err(e) => {
                            ic_cdk::println!("⚠️ Could not confirm Permit2 approval: {}", e);
                            ic_cdk::println!("⚠️ Continuing anyway - swap will fail if approval is missing");
                        }
                    }
                }
                Err(e) => {
                    ic_cdk::println!("⚠️ Permit2 approval transaction error: {}", e);
                    ic_cdk::println!("⚠️ Will attempt swap anyway (might have existing approval)");
                }
            }
        }

        // Step 3: Build V3 swap path
        let path = universal_router::encode_v3_path(
            vec![actual_token_in, actual_token_out],
            vec![fee_tier],
        );

        // Step 4: Execute the V3 swap
        cmd_list.push(commands::V3_SWAP_EXACT_IN);

        // Determine recipient for swap output
        let swap_recipient = if needs_unwrap {
            // If we need to unwrap, send to router (ADDRESS_THIS)
            special_addresses::ADDRESS_THIS.parse().unwrap()
        } else {
            // Send directly to canister
            recipient
        };

        let swap_input = universal_router::encode_v3_swap_exact_in(
            swap_recipient,
            U256::from(amount_in),
            U256::from(min_amount_out),
            path,
            !needs_wrap, // payerIsUser = true if not wrapping (tokens come from msg.sender)
        );
        input_list.push(swap_input);

        // Step 5: If ETH output, unwrap WETH to ETH
        if needs_unwrap {
            cmd_list.push(commands::UNWRAP_WETH);
            input_list.push(universal_router::encode_unwrap_weth(recipient, U256::from(min_amount_out)));
        }

        // Build the complete execute() calldata
        let execute_calldata = universal_router::encode_execute(
            cmd_list,
            input_list,
            deadline,
        );

        // Determine ETH value to send
        let eth_value = if needs_wrap { amount_in } else { 0 };

        // Execute via Universal Router
        match evm_executor.call_contract(chain, router_address, execute_calldata, eth_value).await {
            Ok(tx_hash) => ExecutionResult {
                success: true,
                chain: chain.to_string(),
                tx_hash: Some(tx_hash),
                error: None,
            },
            Err(e) => ExecutionResult {
                success: false,
                chain: chain.to_string(),
                tx_hash: None,
                error: Some(format!("Universal Router swap failed: {}", e)),
            },
        }
    }

    /// Execute a token approval
    async fn execute_approve(
        &self,
        chain: &str,
        token: &str,
        spender: &str,
        amount: u64,
    ) -> ExecutionResult {
        use crate::abi::erc20;
        use ethers_core::types::{Address, U256};

        // Parse spender address
        let spender_addr: Address = match spender.parse() {
            Ok(addr) => addr,
            Err(e) => {
                return ExecutionResult {
                    success: false,
                    chain: chain.to_string(),
                    tx_hash: None,
                    error: Some(format!("Invalid spender address: {:?}", e)),
                }
            }
        };

        // Encode approve(spender, amount) call data
        let amount_u256 = U256::from(amount);
        let call_data = erc20::encode_approve(spender_addr, amount_u256);

        // Create EVM RPC executor
        let evm_executor = match EvmRpcExecutor::new(
            self.key_name.clone(),
            self.derivation_path.clone(),
        ) {
            Ok(executor) => executor,
            Err(e) => {
                return ExecutionResult {
                    success: false,
                    chain: chain.to_string(),
                    tx_hash: None,
                    error: Some(format!("Failed to create EVM RPC executor: {}", e)),
                }
            }
        };

        // Execute approve via contract call (no ETH value sent)
        match evm_executor.call_contract(chain, token, call_data, 0).await {
            Ok(tx_hash) => ExecutionResult {
                success: true,
                chain: chain.to_string(),
                tx_hash: Some(tx_hash),
                error: None,
            },
            Err(e) => ExecutionResult {
                success: false,
                chain: chain.to_string(),
                tx_hash: None,
                error: Some(format!("Approval failed: {}", e)),
            },
        }
    }

    // Removed: get_rpc_service - no longer needed with EVM RPC canister approach

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
        // Use test_key_1 for IC testnet/mainnet testing
        // For production, use "key_1"
        Self::new("test_key_1".to_string())
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
