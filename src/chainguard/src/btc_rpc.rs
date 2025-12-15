use crate::btc_address::BitcoinAddress;
use crate::errors::ChainGuardError;
use candid::{CandidType, Deserialize, Principal};
// Use the correct bitcoin_canister API (same as DFINITY examples)
use ic_cdk::bitcoin_canister::{
    bitcoin_get_balance, bitcoin_get_current_fee_percentiles, bitcoin_get_utxos,
    bitcoin_send_transaction, GetBalanceRequest, GetCurrentFeePercentilesRequest,
    GetUtxosRequest, MillisatoshiPerByte, Network, Outpoint as IcOutpoint, SendTransactionRequest,
    Utxo as IcUtxo,
};
use serde::Serialize;

/// Bitcoin Canister IDs
const BITCOIN_MAINNET_CANISTER: &str = "ghsi2-tqaaa-aaaan-aaaca-cai";
const BITCOIN_TESTNET_CANISTER: &str = "g4xu7-jiaaa-aaaan-aaaaq-cai";

// REMOVED: Blockstream API types - No longer needed after refactoring to bitcoin_canister helpers
// (BlockstreamTransaction, BlockstreamVout, BlockstreamUtxo, BlockstreamUtxoStatus)

/// Bitcoin address types supported
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub enum BitcoinAddressType {
    P2PKH,   // Legacy (Pay-to-Public-Key-Hash)
    P2WPKH,  // SegWit v0 (Pay-to-Witness-Public-Key-Hash)
    P2TR,    // Taproot (Pay-to-Taproot)
}

/// UTXO representation
#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct Utxo {
    pub outpoint: Outpoint,
    pub value: u64,      // satoshis
    pub height: u32,     // block height
}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct Outpoint {
    pub txid: Vec<u8>,   // transaction ID (32 bytes)
    pub vout: u32,       // output index
}

/// Bitcoin transaction builder
#[derive(Debug, Clone)]
pub struct BitcoinTransaction {
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub locktime: u32,
}

#[derive(Debug, Clone)]
pub struct TxInput {
    pub previous_output: Outpoint,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
    pub witness: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct TxOutput {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
}

/// Bitcoin RPC Executor
pub struct BtcRpcExecutor {
    network: Network,
}

impl BtcRpcExecutor {
    /// Create new Bitcoin RPC executor
    pub fn new(chain: &str) -> Result<Self, ChainGuardError> {
        let network = match chain {
            "Bitcoin" => Network::Mainnet,
            "BitcoinTestnet" => Network::Testnet,
            _ => {
                return Err(ChainGuardError::UnsupportedChain {
                    msg: "Only Bitcoin and BitcoinTestnet are supported".to_string(),
                })
            }
        };

        Ok(Self { network })
    }

    /// Get Bitcoin canister ID based on network
    fn get_canister_id(&self) -> Result<Principal, ChainGuardError> {
        let canister_str = match self.network {
            Network::Mainnet => BITCOIN_MAINNET_CANISTER,
            Network::Testnet => BITCOIN_TESTNET_CANISTER,
            _ => {
                return Err(ChainGuardError::UnsupportedChain {
                    msg: "Regtest not supported in production".to_string(),
                })
            }
        };

        Principal::from_text(canister_str)
            .map_err(|e| ChainGuardError::InvalidInput { msg: format!("Invalid canister ID: {}", e) })
    }

    // REMOVED: get_utxos_from_blockstream() - No longer needed, using bitcoin_canister helpers
    // REMOVED: verify_utxo_exists() - No longer needed, using bitcoin_canister helpers

    /// Get UTXOs for a Bitcoin address using Bitcoin canister API (same as DFINITY examples)
    pub async fn get_utxos(&self, address: &str) -> Result<Vec<Utxo>, ChainGuardError> {
        ic_cdk::println!("🔍 [UTXO-FETCH] Starting get_utxos() for address: {}", address);

        // Use bitcoin_get_utxos helper function (same as DFINITY examples)
        let response = bitcoin_get_utxos(&GetUtxosRequest {
            address: address.to_string(),
            network: self.network.clone(),
            filter: None,
        })
        .await
        .map_err(|e| {
            ChainGuardError::ExecutionFailed {
                reason: format!("Failed to get UTXOs from Bitcoin canister: {:?}", e),
            }
        })?;

        ic_cdk::println!("Bitcoin canister returned {} UTXOs", response.utxos.len());

        // Convert Bitcoin canister UTXOs
        let valid_utxos: Vec<Utxo> = response
            .utxos
            .into_iter()
            .map(|ic_utxo| {
                Utxo {
                    outpoint: Outpoint {
                        txid: ic_utxo.outpoint.txid,
                        vout: ic_utxo.outpoint.vout,
                    },
                    value: ic_utxo.value,
                    height: ic_utxo.height,
                }
            })
            .collect();

        ic_cdk::println!("✅ Using {} UTXOs from Bitcoin Canister", valid_utxos.len());
        Ok(valid_utxos)
    }

    /// Get balance for a Bitcoin address
    pub async fn get_balance(&self, address: &str) -> Result<u64, ChainGuardError> {
        let balance = bitcoin_get_balance(&GetBalanceRequest {
            address: address.to_string(),
            network: self.network.clone(),
            min_confirmations: Some(1),
        })
        .await
        .map_err(|e| {
            ChainGuardError::ExecutionFailed { reason: format!("Failed to get balance: {:?}", e) }
        })?;

        Ok(balance)
    }

    /// Get current fee percentiles (returns 101 values from 0-100th percentile)
    pub async fn get_fee_percentiles(&self) -> Result<Vec<MillisatoshiPerByte>, ChainGuardError> {
        let percentiles = bitcoin_get_current_fee_percentiles(&GetCurrentFeePercentilesRequest {
            network: self.network.clone(),
        })
        .await
        .map_err(|e| {
            ChainGuardError::ExecutionFailed {
                reason: format!("Failed to get fees: {:?}", e),
            }
        })?;

        Ok(percentiles)
    }

    /// Estimate fee for a transaction (uses median - 50th percentile)
    pub async fn estimate_fee(&self, tx_size_bytes: u64) -> Result<u64, ChainGuardError> {
        let percentiles = self.get_fee_percentiles().await?;

        // Use median (50th percentile)
        let median_fee_rate = percentiles
            .get(50)
            .ok_or_else(|| ChainGuardError::ExecutionFailed { reason: "No fee data available".to_string() })?;

        // Convert millisatoshi/vbyte to satoshi for total tx
        // 1 sat = 1000 millisat
        let fee_satoshis = (median_fee_rate * tx_size_bytes) / 1000;

        Ok(fee_satoshis)
    }

    /// Select UTXOs to cover amount + fees (simple greedy algorithm)
    pub fn select_utxos(
        &self,
        available_utxos: &[Utxo],
        target_amount: u64,
        estimated_fee: u64,
    ) -> Result<Vec<Utxo>, ChainGuardError> {
        let total_needed = target_amount + estimated_fee;
        let mut selected = Vec::new();
        let mut total_selected = 0u64;

        // Sort UTXOs by value descending (greedy selection)
        let mut sorted_utxos = available_utxos.to_vec();
        sorted_utxos.sort_by(|a, b| b.value.cmp(&a.value));

        for utxo in sorted_utxos {
            if total_selected >= total_needed {
                break;
            }
            total_selected += utxo.value;
            selected.push(utxo);
        }

        if total_selected < total_needed {
            return Err(ChainGuardError::InsufficientFunds {
                msg: format!(
                    "Need {} satoshis, only have {}",
                    total_needed, total_selected
                ),
            });
        }

        Ok(selected)
    }

    /// Build a Bitcoin transaction
    fn build_transaction(
        &self,
        inputs: Vec<Utxo>,
        recipient: &str,
        amount: u64,
        change_address: &str,
        fee: u64,
    ) -> Result<BitcoinTransaction, ChainGuardError> {
        // Calculate total input value
        let total_input: u64 = inputs.iter().map(|utxo| utxo.value).sum();
        let total_output = amount + fee;
        let change = total_input
            .checked_sub(total_output)
            .ok_or_else(|| ChainGuardError::InsufficientFunds { msg: "Insufficient funds".to_string() })?;

        // Build transaction inputs
        let tx_inputs: Vec<TxInput> = inputs
            .iter()
            .map(|utxo| TxInput {
                previous_output: utxo.outpoint.clone(),
                script_sig: Vec::new(), // Will be filled during signing
                sequence: 0xffffffff,   // Standard sequence
                witness: Vec::new(),    // Will be filled for SegWit
            })
            .collect();

        // Build transaction outputs
        let mut tx_outputs = vec![TxOutput {
            value: amount,
            script_pubkey: self.address_to_script_pubkey(recipient)?,
        }];

        // Add change output if change is significant (>= 546 satoshis dust limit)
        if change >= 546 {
            tx_outputs.push(TxOutput {
                value: change,
                script_pubkey: self.address_to_script_pubkey(change_address)?,
            });
        }

        Ok(BitcoinTransaction {
            inputs: tx_inputs,
            outputs: tx_outputs,
            locktime: 0,
        })
    }

    /// Convert address to scriptPubKey
    fn address_to_script_pubkey(&self, address: &str) -> Result<Vec<u8>, ChainGuardError> {
        BitcoinAddress::address_to_script_pubkey(address)
    }

    /// Serialize transaction to raw bytes for signing
    pub fn serialize_transaction(&self, tx: &BitcoinTransaction) -> Vec<u8> {
        let mut serialized = Vec::new();

        // Version (4 bytes, little-endian)
        serialized.extend_from_slice(&2u32.to_le_bytes());

        // Input count (varint)
        serialized.push(tx.inputs.len() as u8);

        // Inputs
        for input in &tx.inputs {
            // Previous outpoint (36 bytes)
            serialized.extend_from_slice(&input.previous_output.txid);
            serialized.extend_from_slice(&input.previous_output.vout.to_le_bytes());

            // Script length and script
            serialized.push(input.script_sig.len() as u8);
            serialized.extend_from_slice(&input.script_sig);

            // Sequence (4 bytes)
            serialized.extend_from_slice(&input.sequence.to_le_bytes());
        }

        // Output count (varint)
        serialized.push(tx.outputs.len() as u8);

        // Outputs
        for output in &tx.outputs {
            // Value (8 bytes, little-endian)
            serialized.extend_from_slice(&output.value.to_le_bytes());

            // Script length and script
            serialized.push(output.script_pubkey.len() as u8);
            serialized.extend_from_slice(&output.script_pubkey);
        }

        // Locktime (4 bytes)
        serialized.extend_from_slice(&tx.locktime.to_le_bytes());

        serialized
    }

    /// Send Bitcoin transaction (using bitcoin_send_transaction helper - same as DFINITY examples)
    pub async fn send_transaction(&self, tx_bytes: Vec<u8>) -> Result<String, ChainGuardError> {
        ic_cdk::println!("=== Sending Transaction to Bitcoin Canister ===");
        ic_cdk::println!("Network: {:?}", self.network);
        ic_cdk::println!("TX bytes length: {}", tx_bytes.len());
        ic_cdk::println!("TX hex: {}", hex::encode(&tx_bytes));

        // Use bitcoin_send_transaction helper function (same as DFINITY examples)
        bitcoin_send_transaction(&SendTransactionRequest {
            network: self.network.clone(),
            transaction: tx_bytes,
        })
        .await
        .map_err(|e| {
            ic_cdk::println!("❌ Bitcoin canister rejected transaction: {:?}", e);
            ic_cdk::println!("=== End Send Transaction ===");
            ChainGuardError::ExecutionFailed {
                reason: format!("Bitcoin canister rejected transaction: {:?}", e),
            }
        })?;

        ic_cdk::println!("✅ Bitcoin canister accepted the transaction");
        ic_cdk::println!("⏳ Transaction queued for propagation by Bitcoin adapters");
        ic_cdk::println!("📝 Note: Propagation is asynchronous - may take 5-10 minutes to appear on block explorers");
        ic_cdk::println!("=== End Send Transaction ===");

        Ok("Transaction submitted successfully - propagation in progress".to_string())
    }

    // REMOVED: broadcast_to_blockstream() - No longer needed, using bitcoin_send_transaction helper

    /// Execute a Bitcoin transfer (high-level method) using rust-bitcoin
    pub async fn transfer(
        &self,
        from_address: &str,
        to_address: &str,
        amount: u64,
        key_name: String,
        derivation_path: Vec<Vec<u8>>,
        _public_key: &[u8], // Not used, we get it from Chain-Key
    ) -> Result<String, ChainGuardError> {
        use crate::btc_signing::sign_p2wpkh_transaction;
        use crate::btc_transaction::{build_transaction_with_fee, get_fee_per_vbyte, parse_address};

        // 1. Get available UTXOs from Bitcoin canister (testnet4 compatible)
        // ICP's Network::Testnet maps to testnet4, Bitcoin Canister has built-in testnet4 support
        ic_cdk::println!("🔍 Getting UTXOs from Bitcoin Canister");
        ic_cdk::println!("Network: {:?}", self.network);
        let utxos_vec = self.get_utxos(from_address).await?;

        if utxos_vec.is_empty() {
            return Err(ChainGuardError::InsufficientFunds {
                msg: "No UTXOs available".to_string(),
            });
        }

        ic_cdk::println!("✅ Found {} valid UTXOs", utxos_vec.len());

        // Convert from our Utxo format to IcUtxo format for build_transaction_with_fee
        let utxos: Vec<IcUtxo> = utxos_vec
            .into_iter()
            .map(|u| IcUtxo {
                outpoint: IcOutpoint {
                    txid: u.outpoint.txid,
                    vout: u.outpoint.vout,
                },
                value: u.value,
                height: u.height,
            })
            .collect();

        // 2. Parse addresses using rust-bitcoin (testnet4 is compatible with testnet)
        // rust-bitcoin's Network::Testnet works for testnet4 address validation
        let btc_network = match self.network {
            Network::Mainnet => bitcoin::Network::Bitcoin,
            Network::Testnet => bitcoin::Network::Testnet,  // Works for testnet4
            Network::Regtest => bitcoin::Network::Regtest,
        };

        let own_addr = parse_address(from_address, btc_network)?;
        let dst_addr = parse_address(to_address, btc_network)?;

        // 3. Get fee estimate using ICP's fee API
        let old_network = match self.network {
            Network::Mainnet => ic_cdk::api::management_canister::bitcoin::BitcoinNetwork::Mainnet,
            Network::Testnet => ic_cdk::api::management_canister::bitcoin::BitcoinNetwork::Testnet,
            Network::Regtest => ic_cdk::api::management_canister::bitcoin::BitcoinNetwork::Regtest,
        };
        let fee_per_vbyte = get_fee_per_vbyte(old_network).await?;

        // Estimate 140 vbytes for P2WPKH transaction
        let estimated_fee = fee_per_vbyte * 140;

        // 4. Build unsigned transaction
        let (unsigned_tx, prev_outputs) =
            build_transaction_with_fee(&own_addr, &utxos, &dst_addr, amount, estimated_fee)?;

        // 5. Sign transaction with Chain-Key ECDSA
        let signed_tx = sign_p2wpkh_transaction(
            unsigned_tx,
            &own_addr,
            &prev_outputs,
            key_name,
            derivation_path,
        )
        .await?;

        // 6. Serialize transaction
        let tx_bytes = bitcoin::consensus::encode::serialize(&signed_tx);

        ic_cdk::println!("📦 Transaction size: {} bytes", tx_bytes.len());
        ic_cdk::println!("📤 Sending transaction to Bitcoin network via IC Bitcoin Canister");

        // 7. Send transaction FIRST (following DFINITY pattern)
        self.send_transaction(tx_bytes).await?;

        // 8. Compute txid AFTER sending (following DFINITY pattern)
        let txid = signed_tx.compute_txid().to_string();

        ic_cdk::println!("✅ Transaction sent successfully");
        ic_cdk::println!("TXID: {}", txid);

        // Return the txid
        Ok(txid)
    }

    /// Calculate pubkey hash directly from public key
    /// This is HASH160(pubkey) = RIPEMD160(SHA256(pubkey))
    fn calculate_pubkey_hash(&self, public_key: &[u8]) -> Result<Vec<u8>, ChainGuardError> {
        use ripemd::Ripemd160;
        use sha2::{Digest, Sha256};

        // Compress public key if needed (must be 33 bytes for SegWit)
        let compressed_pubkey = if public_key.len() == 65 {
            // Uncompressed key: 0x04 + x (32 bytes) + y (32 bytes)
            // Compressed: 0x02/0x03 + x (32 bytes) based on y parity
            let mut compressed = vec![if public_key[64] % 2 == 0 { 0x02 } else { 0x03 }];
            compressed.extend_from_slice(&public_key[1..33]);
            compressed
        } else if public_key.len() == 33 {
            // Already compressed
            public_key.to_vec()
        } else {
            return Err(ChainGuardError::InvalidInput {
                msg: format!("Invalid public key length: {}", public_key.len()),
            });
        };

        // HASH160 = RIPEMD160(SHA256(compressed_pubkey))
        let sha256_hash = Sha256::digest(&compressed_pubkey);
        let ripemd_hash = Ripemd160::digest(&sha256_hash);

        Ok(ripemd_hash.to_vec())
    }

    /// Extract pubkey hash from Bitcoin address (for P2WPKH)
    /// NOTE: This is now deprecated in favor of calculate_pubkey_hash
    fn extract_pubkey_hash_from_address(&self, address: &str) -> Result<Vec<u8>, ChainGuardError> {
        if address.starts_with("bc1") || address.starts_with("tb1") {
            // Bech32 decode
            let (_hrp, data, _variant) = bech32::decode(address).map_err(|e| {
                ChainGuardError::InvalidInput {
                    msg: format!("Failed to decode bech32 address: {}", e),
                }
            })?;

            // Skip witness version (first element), convert rest from base32 to bytes
            if data.is_empty() {
                return Err(ChainGuardError::InvalidInput {
                    msg: "Empty Bech32 data".to_string(),
                });
            }

            let data_u8: Vec<u8> = data[1..].iter().map(|u5| u5.to_u8()).collect();
            let decoded = Self::convert_bits(&data_u8, 5, 8, false)?;

            // For P2WPKH, decoded data should be 20 bytes (pubkey hash)
            if decoded.len() == 20 {
                Ok(decoded)
            } else {
                Err(ChainGuardError::InvalidInput {
                    msg: format!("Invalid pubkey hash length: {} (expected 20)", decoded.len()),
                })
            }
        } else {
            Err(ChainGuardError::InvalidInput {
                msg: "Only SegWit addresses supported for hash extraction".to_string(),
            })
        }
    }

    /// Convert base32 (5-bit) to bytes (8-bit) manually
    /// This avoids the padding issues in bech32 0.9's FromBase32
    fn convert_bits(data: &[u8], from_bits: u32, to_bits: u32, pad: bool) -> Result<Vec<u8>, ChainGuardError> {
        let mut acc: u32 = 0;
        let mut bits: u32 = 0;
        let mut ret = Vec::new();
        let maxv: u32 = (1 << to_bits) - 1;
        let max_acc: u32 = (1 << (from_bits + to_bits - 1)) - 1;

        for value in data {
            let value = *value as u32;
            if value >> from_bits != 0 {
                return Err(ChainGuardError::InvalidInput {
                    msg: "Invalid data for convert_bits".to_string(),
                });
            }
            acc = ((acc << from_bits) | value) & max_acc;
            bits += from_bits;
            while bits >= to_bits {
                bits -= to_bits;
                ret.push(((acc >> bits) & maxv) as u8);
            }
        }

        if pad {
            if bits > 0 {
                ret.push(((acc << (to_bits - bits)) & maxv) as u8);
            }
        } else {
            // For non-padded conversion, check if leftover bits are all zeros
            if bits > 0 {
                // Calculate the padded value
                let padded = (acc << (to_bits - bits)) & maxv;
                if padded != 0 {
                    return Err(ChainGuardError::InvalidInput {
                        msg: "Invalid padding in convert_bits: non-zero padding bits".to_string(),
                    });
                }
            }
        }

        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utxo_selection() {
        let executor = BtcRpcExecutor {
            network: Network::Testnet,
        };

        let utxos = vec![
            Utxo {
                outpoint: Outpoint {
                    txid: vec![0u8; 32],
                    vout: 0,
                },
                value: 100000,
                height: 100,
            },
            Utxo {
                outpoint: Outpoint {
                    txid: vec![1u8; 32],
                    vout: 0,
                },
                value: 50000,
                height: 101,
            },
            Utxo {
                outpoint: Outpoint {
                    txid: vec![2u8; 32],
                    vout: 0,
                },
                value: 25000,
                height: 102,
            },
        ];

        // Test selecting enough UTXOs
        let selected = executor.select_utxos(&utxos, 120000, 5000).unwrap();
        assert_eq!(selected.len(), 2); // Should select 100k and 50k UTXOs

        let total: u64 = selected.iter().map(|u| u.value).sum();
        assert!(total >= 125000);
    }

    #[test]
    fn test_insufficient_funds() {
        let executor = BtcRpcExecutor {
            network: Network::Testnet,
        };

        let utxos = vec![Utxo {
            outpoint: Outpoint {
                txid: vec![0u8; 32],
                vout: 0,
            },
            value: 10000,
            height: 100,
        }];

        // Try to send more than available
        let result = executor.select_utxos(&utxos, 50000, 1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_transaction() {
        let executor = BtcRpcExecutor {
            network: Network::Testnet,
        };

        let utxos = vec![Utxo {
            outpoint: Outpoint {
                txid: vec![1u8; 32],
                vout: 0,
            },
            value: 100000,
            height: 100,
        }];

        let tx = executor
            .build_transaction(
                utxos,
                "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx", // Testnet P2WPKH
                50000,
                "tb1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3q0sl5k7", // Change address
                1000,
            )
            .unwrap();

        assert_eq!(tx.inputs.len(), 1);
        assert_eq!(tx.outputs.len(), 2); // Payment + change
        assert_eq!(tx.outputs[0].value, 50000);
        assert_eq!(tx.outputs[1].value, 49000); // 100000 - 50000 - 1000 fee
    }

    #[test]
    fn test_extract_pubkey_hash() {
        let executor = BtcRpcExecutor {
            network: Network::Testnet,
        };

        // Test P2WPKH address
        let address = "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx";
        let result = executor.extract_pubkey_hash_from_address(address);
        assert!(result.is_ok());

        let hash = result.unwrap();
        assert_eq!(hash.len(), 20); // HASH160 is 20 bytes
    }

    #[test]
    fn test_transaction_serialization() {
        let tx = BitcoinTransaction {
            inputs: vec![TxInput {
                previous_output: Outpoint {
                    txid: vec![0xaa; 32],
                    vout: 1,
                },
                script_sig: Vec::new(),
                sequence: 0xffffffff,
                witness: Vec::new(),
            }],
            outputs: vec![TxOutput {
                value: 50000,
                script_pubkey: vec![0x00, 0x14, 0xaa; 20], // P2WPKH
            }],
            locktime: 0,
        };

        let executor = BtcRpcExecutor {
            network: Network::Testnet,
        };

        let serialized = executor.serialize_transaction(&tx);

        // Check version (first 4 bytes)
        assert_eq!(&serialized[0..4], &2u32.to_le_bytes());

        // Check input count
        assert_eq!(serialized[4], 1);

        // Verify structure is valid (no panics)
        assert!(serialized.len() > 50); // Reasonable minimum size
    }
}
