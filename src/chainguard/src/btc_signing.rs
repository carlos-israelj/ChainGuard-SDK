/// Bitcoin transaction signing using Chain-Key ECDSA and rust-bitcoin
/// Based on DFINITY's basic_bitcoin example
use bitcoin::{
    ecdsa::Signature as BitcoinSignature,
    hashes::Hash,
    key::CompressedPublicKey,
    sighash::{EcdsaSighashType, SighashCache},
    secp256k1::{ecdsa::Signature as Secp256k1Signature, PublicKey as Secp256k1PublicKey},
    Address, PublicKey, Transaction, TxOut,
};
use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, sign_with_ecdsa, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument,
    SignWithEcdsaArgument,
};
use std::cell::RefCell;

use crate::errors::ChainGuardError;

thread_local! {
    /// Cache for ECDSA public key to avoid repeated calls
    static ECDSA_PUBLIC_KEY_CACHE: RefCell<Option<Vec<u8>>> = RefCell::new(None);
}

/// Get ECDSA public key from Chain-Key (with caching)
pub async fn get_ecdsa_public_key_cached(
    key_name: String,
    derivation_path: Vec<Vec<u8>>,
) -> Result<Vec<u8>, ChainGuardError> {
    // Check cache first
    let cached = ECDSA_PUBLIC_KEY_CACHE.with(|cache| cache.borrow().clone());
    if let Some(pubkey) = cached {
        return Ok(pubkey);
    }

    // Call management canister
    let key_id = EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: key_name,
    };

    let args = EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path,
        key_id,
    };

    let (response,) = ecdsa_public_key(args)
        .await
        .map_err(|(code, msg)| ChainGuardError::ExecutionFailed {
            reason: format!("Failed to get ECDSA public key: {:?} - {}", code, msg),
        })?;

    // Cache the result
    ECDSA_PUBLIC_KEY_CACHE.with(|cache| {
        *cache.borrow_mut() = Some(response.public_key.clone());
    });

    Ok(response.public_key)
}

/// Sign a message hash with Chain-Key ECDSA
async fn sign_with_ecdsa_internal(
    key_name: String,
    derivation_path: Vec<Vec<u8>>,
    message_hash: Vec<u8>,
) -> Result<Vec<u8>, ChainGuardError> {
    let key_id = EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: key_name,
    };

    let args = SignWithEcdsaArgument {
        message_hash,
        derivation_path,
        key_id,
    };

    let (response,) = sign_with_ecdsa(args)
        .await
        .map_err(|(code, msg)| ChainGuardError::ExecutionFailed {
            reason: format!("Failed to sign with ECDSA: {:?} - {}", code, msg),
        })?;

    // ICP returns 64-byte signature (r || s)
    Ok(response.signature)
}

/// Sign a P2WPKH transaction with Chain-Key ECDSA
pub async fn sign_p2wpkh_transaction(
    mut transaction: Transaction,
    own_address: &Address,
    prev_outputs: &[TxOut],
    key_name: String,
    derivation_path: Vec<Vec<u8>>,
) -> Result<Transaction, ChainGuardError> {
    // Validate address is P2WPKH (witness version 0)
    // P2WPKH addresses start with bc1q (mainnet) or tb1q (testnet)
    let addr_str = own_address.to_string();
    if !addr_str.starts_with("bc1q") && !addr_str.starts_with("tb1q") {
        return Err(ChainGuardError::InvalidInput {
            msg: format!("Only P2WPKH addresses supported, got: {}", addr_str),
        });
    }

    // Get public key
    let pubkey_bytes = get_ecdsa_public_key_cached(key_name.clone(), derivation_path.clone()).await?;

    // Parse public key
    let secp_pubkey = Secp256k1PublicKey::from_slice(&pubkey_bytes)
        .map_err(|e| ChainGuardError::ExecutionFailed {
            reason: format!("Invalid public key: {}", e),
        })?;

    let pubkey = PublicKey::new(secp_pubkey);

    // Create sighash cache
    let mut sighash_cache = SighashCache::new(&transaction);

    // Sign each input
    let mut signatures = Vec::new();
    for (index, prev_output) in prev_outputs.iter().enumerate() {
        // Compute sighash for this input
        let sighash = sighash_cache
            .p2wpkh_signature_hash(
                index,
                &prev_output.script_pubkey,
                prev_output.value,
                EcdsaSighashType::All,
            )
            .map_err(|e| ChainGuardError::ExecutionFailed {
                reason: format!("Failed to compute sighash: {}", e),
            })?;

        ic_cdk::println!("🔐 Signing input {} with sighash: {}", index, hex::encode(sighash.as_byte_array()));

        // Sign with Chain-Key ECDSA
        let signature_bytes = sign_with_ecdsa_internal(
            key_name.clone(),
            derivation_path.clone(),
            sighash.as_byte_array().to_vec(),
        )
        .await?;

        ic_cdk::println!("✅ Received signature: {}", hex::encode(&signature_bytes));

        // Convert to bitcoin signature
        let secp_sig = Secp256k1Signature::from_compact(&signature_bytes)
            .map_err(|e| ChainGuardError::ExecutionFailed {
                reason: format!("Invalid signature format: {}", e),
            })?;

        let bitcoin_sig = BitcoinSignature::sighash_all(secp_sig);

        signatures.push(bitcoin_sig);
    }

    // Finalize transaction with witness data
    let final_tx = sighash_cache.into_transaction();
    let mut final_tx = final_tx.clone();

    for (index, signature) in signatures.iter().enumerate() {
        let mut witness = bitcoin::Witness::new();
        witness.push(signature.to_vec());
        witness.push(pubkey.to_bytes());
        final_tx.input[index].witness = witness;
    }

    ic_cdk::println!("✅ Transaction signed successfully");

    Ok(final_tx)
}

/// Get Bitcoin address from ECDSA public key (P2WPKH)
pub async fn get_p2wpkh_address(
    key_name: String,
    derivation_path: Vec<Vec<u8>>,
    network: bitcoin::Network,
) -> Result<String, ChainGuardError> {
    let pubkey_bytes = get_ecdsa_public_key_cached(key_name, derivation_path).await?;

    let secp_pubkey = Secp256k1PublicKey::from_slice(&pubkey_bytes)
        .map_err(|e| ChainGuardError::ExecutionFailed {
            reason: format!("Invalid public key: {}", e),
        })?;

    // Create CompressedPublicKey directly from secp pubkey
    let compressed_pubkey = CompressedPublicKey(secp_pubkey);

    // Create P2WPKH address
    let address = Address::p2wpkh(&compressed_pubkey, network);

    Ok(address.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_key_parsing() {
        // Valid compressed public key (33 bytes)
        let pubkey_hex = "02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5";
        let pubkey_bytes = hex::decode(pubkey_hex).unwrap();

        let result = Secp256k1PublicKey::from_slice(&pubkey_bytes);
        assert!(result.is_ok());
    }
}
