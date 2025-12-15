/// Bitcoin transaction building utilities using rust-bitcoin
/// Based on DFINITY's basic_bitcoin example
use bitcoin::{
    absolute::LockTime,
    hashes::Hash,
    transaction::{Transaction, Version},
    Address, Amount, OutPoint, ScriptBuf, Sequence, TxIn, TxOut, Txid,
};
// Use the correct bitcoin_canister API (same as DFINITY examples)
use ic_cdk::bitcoin_canister::{MillisatoshiPerByte, Utxo as IcUtxo};
use std::str::FromStr;

use crate::errors::ChainGuardError;

/// Select UTXOs using greedy algorithm to cover amount + fee
pub fn select_utxos_greedy(
    utxos: Vec<IcUtxo>,
    amount: u64,
    fee: u64,
) -> Result<Vec<IcUtxo>, ChainGuardError> {
    let mut selected_utxos = Vec::new();
    let mut total = 0u64;
    let target = amount + fee;

    // Sort UTXOs by value descending (greedy selection)
    let mut sorted_utxos = utxos;
    sorted_utxos.sort_by(|a, b| b.value.cmp(&a.value));

    for utxo in sorted_utxos {
        if total >= target {
            break;
        }
        total += utxo.value;
        selected_utxos.push(utxo);
    }

    if total < target {
        return Err(ChainGuardError::InsufficientFunds {
            msg: format!("Need {} satoshis, only have {}", target, total),
        });
    }

    Ok(selected_utxos)
}

/// Build unsigned Bitcoin transaction with fee calculation
pub fn build_transaction_with_fee(
    own_address: &Address,
    own_utxos: &[IcUtxo],
    dst_address: &Address,
    amount: u64,
    fee: u64,
) -> Result<(Transaction, Vec<TxOut>), ChainGuardError> {
    // Select UTXOs to cover amount + fee
    let selected_utxos = select_utxos_greedy(own_utxos.to_vec(), amount, fee)?;

    // Calculate total input value
    let total_input: u64 = selected_utxos.iter().map(|utxo| utxo.value).sum();

    // Calculate change
    let change = total_input
        .checked_sub(amount + fee)
        .ok_or_else(|| ChainGuardError::InsufficientFunds {
            msg: "Insufficient funds for transaction".to_string(),
        })?;

    // Build transaction inputs
    let mut inputs = Vec::new();
    let mut prev_outputs = Vec::new();

    for utxo in &selected_utxos {
        // Convert txid bytes to Txid using from_slice
        let mut txid_bytes = [0u8; 32];
        txid_bytes.copy_from_slice(&utxo.outpoint.txid);
        let hash = bitcoin::hashes::sha256d::Hash::from_slice(&txid_bytes)
            .expect("32 bytes is valid for sha256d");
        let txid = Txid::from_raw_hash(hash);

        inputs.push(TxIn {
            previous_output: OutPoint {
                txid,
                vout: utxo.outpoint.vout,
            },
            script_sig: ScriptBuf::new(), // Empty for SegWit
            sequence: Sequence::MAX,       // Enable RBF
            witness: bitcoin::Witness::new(), // Will be filled during signing
        });

        // Create previous output for signing (needed for P2WPKH)
        prev_outputs.push(TxOut {
            value: Amount::from_sat(utxo.value),
            script_pubkey: own_address.script_pubkey(),
        });
    }

    // Build transaction outputs
    let mut outputs = vec![TxOut {
        value: Amount::from_sat(amount),
        script_pubkey: dst_address.script_pubkey(),
    }];

    // Add change output if above dust threshold (1000 sats)
    if change >= 1000 {
        outputs.push(TxOut {
            value: Amount::from_sat(change),
            script_pubkey: own_address.script_pubkey(),
        });
    }

    let transaction = Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: inputs,
        output: outputs,
    };

    Ok((transaction, prev_outputs))
}

/// Get fee per vbyte from Bitcoin canister
pub async fn get_fee_per_vbyte(
    network: ic_cdk::api::management_canister::bitcoin::BitcoinNetwork,
) -> Result<u64, ChainGuardError> {
    use ic_cdk::api::management_canister::bitcoin::{
        bitcoin_get_current_fee_percentiles, GetCurrentFeePercentilesRequest,
    };

    let request = GetCurrentFeePercentilesRequest { network };

    let percentiles: Vec<MillisatoshiPerByte> =
        bitcoin_get_current_fee_percentiles(request)
            .await
            .map_err(|(code, msg)| ChainGuardError::ExecutionFailed {
                reason: format!("Failed to get fee percentiles: {:?} - {}", code, msg),
            })?
            .0;

    // Use median (50th percentile)
    let median_fee_rate = percentiles
        .get(50)
        .ok_or_else(|| ChainGuardError::ExecutionFailed {
            reason: "No fee data available".to_string(),
        })?;

    // Convert millisatoshi/vbyte to satoshi/vbyte
    // 1 sat = 1000 millisat
    Ok(median_fee_rate / 1000)
}

/// Parse Bitcoin address from string
pub fn parse_address(
    address: &str,
    network: bitcoin::Network,
) -> Result<Address, ChainGuardError> {
    Address::from_str(address)
        .map_err(|e| ChainGuardError::InvalidInput {
            msg: format!("Invalid Bitcoin address: {}", e),
        })
        .and_then(|addr| {
            addr.require_network(network)
                .map_err(|e| ChainGuardError::InvalidInput {
                    msg: format!("Address network mismatch: {}", e),
                })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_cdk::api::management_canister::bitcoin::Outpoint as IcOutpoint;

    #[test]
    fn test_select_utxos_greedy() {
        let utxos = vec![
            IcUtxo {
                outpoint: IcOutpoint {
                    txid: vec![0u8; 32],
                    vout: 0,
                },
                value: 100000,
                height: 100,
            },
            IcUtxo {
                outpoint: IcOutpoint {
                    txid: vec![1u8; 32],
                    vout: 0,
                },
                value: 50000,
                height: 101,
            },
        ];

        let selected = select_utxos_greedy(utxos, 120000, 5000).unwrap();
        assert_eq!(selected.len(), 2);

        let total: u64 = selected.iter().map(|u| u.value).sum();
        assert!(total >= 125000);
    }

    #[test]
    fn test_insufficient_funds() {
        let utxos = vec![IcUtxo {
            outpoint: IcOutpoint {
                txid: vec![0u8; 32],
                vout: 0,
            },
            value: 10000,
            height: 100,
        }];

        let result = select_utxos_greedy(utxos, 50000, 1000);
        assert!(result.is_err());
    }
}
