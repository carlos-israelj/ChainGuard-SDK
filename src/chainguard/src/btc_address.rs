use crate::errors::ChainGuardError;
use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument,
};
use sha2::{Digest, Sha256};

/// Bitcoin address derivation utilities
pub struct BitcoinAddress;

impl BitcoinAddress {
    /// Get ECDSA public key for Bitcoin (secp256k1 curve)
    pub async fn get_public_key(
        key_name: String,
        derivation_path: Vec<Vec<u8>>,
    ) -> Result<Vec<u8>, ChainGuardError> {
        let request = EcdsaPublicKeyArgument {
            canister_id: None,
            derivation_path,
            key_id: EcdsaKeyId {
                curve: EcdsaCurve::Secp256k1,
                name: key_name,
            },
        };

        let (response,) = ecdsa_public_key(request)
            .await
            .map_err(|e| {
                ChainGuardError::ExecutionFailed {
                    reason: format!("Failed to get ECDSA public key: {:?}", e),
                }
            })?;

        // The public_key from ICP is DER-encoded
        // We need to extract the actual EC point from the DER structure
        Self::extract_public_key_from_der(&response.public_key)
    }

    /// Extract the EC point from DER-encoded public key
    /// DER format: SEQUENCE { SEQUENCE { OID, OID }, BIT STRING }
    /// We need to extract the BIT STRING which contains 0x04||x||y
    fn extract_public_key_from_der(der_key: &[u8]) -> Result<Vec<u8>, ChainGuardError> {
        // For secp256k1, the DER-encoded public key is approximately 88-91 bytes
        // The actual EC point (0x04||x||y = 65 bytes) is in the BIT STRING at the end

        // Simple extraction: look for 0x04 followed by 64 bytes
        // This is a heuristic approach that works for standard DER encoding
        if let Some(pos) = der_key.iter().position(|&b| b == 0x04) {
            // Check if we have enough bytes after 0x04 for x and y coordinates
            if pos + 65 <= der_key.len() {
                return Ok(der_key[pos..pos + 65].to_vec());
            }
        }

        // Fallback: if the key is already 65 bytes and starts with 0x04, use it directly
        if der_key.len() == 65 && der_key[0] == 0x04 {
            return Ok(der_key.to_vec());
        }

        // Fallback: if it's 33 bytes (compressed), use it directly
        if der_key.len() == 33 && (der_key[0] == 0x02 || der_key[0] == 0x03) {
            return Ok(der_key.to_vec());
        }

        Err(ChainGuardError::InvalidInput {
            msg: format!("Could not extract public key from DER encoding (length: {})", der_key.len()),
        })
    }

    /// Derive P2PKH (Legacy) Bitcoin address from public key
    /// Format: 1Address... (mainnet) or mAddress... (testnet)
    pub fn public_key_to_p2pkh(public_key: &[u8], testnet: bool) -> Result<String, ChainGuardError> {
        // Public key should be 65 bytes (uncompressed) or 33 bytes (compressed)
        if public_key.len() != 65 && public_key.len() != 33 {
            return Err(ChainGuardError::InvalidInput {
                msg: format!("Invalid public key length: {}", public_key.len()),
            });
        }

        // Use compressed public key (33 bytes)
        let compressed_pubkey = if public_key.len() == 65 {
            Self::compress_public_key(public_key)?
        } else {
            public_key.to_vec()
        };

        // 1. SHA256 hash of public key
        let sha256_hash = Sha256::digest(&compressed_pubkey);

        // 2. RIPEMD160 hash of SHA256 hash
        let ripemd_hash = Self::ripemd160(&sha256_hash);

        // 3. Add version byte (0x00 for mainnet, 0x6f for testnet)
        let version = if testnet { 0x6f } else { 0x00 };
        let mut versioned_hash = vec![version];
        versioned_hash.extend_from_slice(&ripemd_hash);

        // 4. Calculate checksum (first 4 bytes of double SHA256)
        let checksum = Self::double_sha256_checksum(&versioned_hash);

        // 5. Append checksum
        versioned_hash.extend_from_slice(&checksum);

        // 6. Base58 encode
        Ok(Self::base58_encode(&versioned_hash))
    }

    /// Derive P2WPKH (SegWit v0) Bitcoin address from public key
    /// Format: bc1q... (mainnet) or tb1q... (testnet)
    pub fn public_key_to_p2wpkh(public_key: &[u8], testnet: bool) -> Result<String, ChainGuardError> {
        use bech32::{ToBase32, Variant, u5};

        // Use compressed public key
        let compressed_pubkey = if public_key.len() == 65 {
            Self::compress_public_key(public_key)?
        } else {
            public_key.to_vec()
        };

        // 1. SHA256 hash
        let sha256_hash = Sha256::digest(&compressed_pubkey);

        // 2. RIPEMD160 hash (this is the witness program)
        let witness_program = Self::ripemd160(&sha256_hash);

        // 3. Convert witness program to base32 (5-bit groups)
        let witness_program_base32 = witness_program.to_base32();

        // 4. Prepend witness version (0 for P2WPKH) as a u5
        let witness_version = u5::try_from_u8(0).map_err(|_| ChainGuardError::ExecutionFailed {
            reason: "Invalid witness version".to_string(),
        })?;

        let mut data = vec![witness_version];
        data.extend(witness_program_base32);

        // 5. Bech32 encode with appropriate HRP
        let hrp = if testnet { "tb" } else { "bc" };
        bech32::encode(hrp, data, Variant::Bech32).map_err(|e| ChainGuardError::ExecutionFailed {
            reason: format!("Bech32 encoding failed: {}", e),
        })
    }

    /// Derive P2TR (Taproot) Bitcoin address from public key
    /// Format: bc1p... (mainnet) or tb1p... (testnet)
    pub fn public_key_to_p2tr(public_key: &[u8], testnet: bool) -> Result<String, ChainGuardError> {
        use bech32::{ToBase32, Variant, u5};

        // For Taproot, we need the x-only public key (32 bytes)
        let x_only_pubkey = if public_key.len() == 65 {
            // Uncompressed: skip prefix byte and y-coordinate
            public_key[1..33].to_vec()
        } else if public_key.len() == 33 {
            // Compressed: skip prefix byte
            public_key[1..].to_vec()
        } else {
            return Err(ChainGuardError::InvalidInput {
                msg: format!("Invalid public key length for Taproot: {}", public_key.len()),
            });
        };

        // Convert witness program to base32
        let witness_program_base32 = x_only_pubkey.to_base32();

        // Prepend witness version 1 for Taproot
        let witness_version = u5::try_from_u8(1).map_err(|_| ChainGuardError::ExecutionFailed {
            reason: "Invalid witness version".to_string(),
        })?;

        let mut data = vec![witness_version];
        data.extend(witness_program_base32);

        // Bech32m encode (Taproot uses Bech32m, not Bech32)
        let hrp = if testnet { "tb" } else { "bc" };
        bech32::encode(hrp, data, Variant::Bech32m).map_err(|e| ChainGuardError::ExecutionFailed {
            reason: format!("Bech32m encoding failed: {}", e),
        })
    }

    /// Compress an uncompressed public key (65 bytes -> 33 bytes)
    fn compress_public_key(pubkey: &[u8]) -> Result<Vec<u8>, ChainGuardError> {
        if pubkey.len() != 65 {
            return Err(ChainGuardError::InvalidInput {
                msg: "Public key must be 65 bytes for compression".to_string(),
            });
        }

        // First byte should be 0x04 for uncompressed
        if pubkey[0] != 0x04 {
            return Err(ChainGuardError::InvalidInput {
                msg: "Invalid uncompressed public key prefix".to_string(),
            });
        }

        let x = &pubkey[1..33];
        let y = &pubkey[33..65];

        // Prefix: 0x02 if y is even, 0x03 if y is odd
        let prefix = if y[31] & 1 == 0 { 0x02 } else { 0x03 };

        let mut compressed = vec![prefix];
        compressed.extend_from_slice(x);

        Ok(compressed)
    }

    /// RIPEMD160 hash (uses external crate or implementation)
    fn ripemd160(data: &[u8]) -> Vec<u8> {
        use ripemd::Ripemd160;
        let mut hasher = Ripemd160::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// Double SHA256 and take first 4 bytes as checksum
    fn double_sha256_checksum(data: &[u8]) -> Vec<u8> {
        let hash1 = Sha256::digest(data);
        let hash2 = Sha256::digest(hash1);
        hash2[0..4].to_vec()
    }

    /// Base58 encode (Bitcoin alphabet)
    fn base58_encode(data: &[u8]) -> String {
        const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

        let mut num = num_bigint::BigUint::from_bytes_be(data);
        let mut encoded = String::new();
        let base = num_bigint::BigUint::from(58u32);

        while num > num_bigint::BigUint::from(0u32) {
            let remainder = &num % &base;
            num /= &base;
            encoded.push(ALPHABET[remainder.to_u32_digits()[0] as usize] as char);
        }

        // Add leading '1's for leading zeros
        for &byte in data {
            if byte == 0 {
                encoded.push('1');
            } else {
                break;
            }
        }

        encoded.chars().rev().collect()
    }

    /// Bech32 encode (SegWit v0)
    fn bech32_encode(hrp: &str, _version: u8, data: &[u8]) -> Result<String, ChainGuardError> {
        use bech32::ToBase32;

        let data_base32 = data.to_base32();
        bech32::encode(hrp, data_base32, bech32::Variant::Bech32)
            .map_err(|e| ChainGuardError::ExecutionFailed {
                reason: format!("Bech32 encoding failed: {}", e),
            })
    }

    /// Bech32m encode (Taproot - SegWit v1+)
    fn bech32m_encode(hrp: &str, version: u8, data: &[u8]) -> Result<String, ChainGuardError> {
        use bech32::{ToBase32, Variant};

        // Prepend witness version to data
        let mut full_data = vec![version];
        full_data.extend_from_slice(data);

        let data_base32 = full_data.to_base32();
        bech32::encode(hrp, data_base32, Variant::Bech32m)
            .map_err(|e| ChainGuardError::ExecutionFailed {
                reason: format!("Bech32m encoding failed: {}", e),
            })
    }

    /// Decode Bitcoin address to scriptPubKey
    pub fn address_to_script_pubkey(address: &str) -> Result<Vec<u8>, ChainGuardError> {
        // Detect address type and convert to scriptPubKey
        if address.starts_with('1') || address.starts_with('m') || address.starts_with('n') {
            // P2PKH address
            Self::p2pkh_address_to_script(address)
        } else if address.starts_with("bc1q") || address.starts_with("tb1q") {
            // P2WPKH address
            Self::p2wpkh_address_to_script(address)
        } else if address.starts_with("bc1p") || address.starts_with("tb1p") {
            // P2TR address
            Self::p2tr_address_to_script(address)
        } else if address.starts_with('3') || address.starts_with('2') {
            // P2SH address (not fully implemented)
            Err(ChainGuardError::NotImplemented {
                feature: "P2SH address decoding".to_string(),
            })
        } else {
            Err(ChainGuardError::InvalidInput {
                msg: format!("Unknown address format: {}", address),
            })
        }
    }

    /// Convert P2PKH address to scriptPubKey
    fn p2pkh_address_to_script(address: &str) -> Result<Vec<u8>, ChainGuardError> {
        let decoded = Self::base58_decode(address)?;

        // Remove version byte and checksum
        if decoded.len() != 25 {
            return Err(ChainGuardError::InvalidInput {
                msg: "Invalid P2PKH address length".to_string(),
            });
        }

        let pubkey_hash = &decoded[1..21];

        // P2PKH scriptPubKey: OP_DUP OP_HASH160 <pubkey_hash> OP_EQUALVERIFY OP_CHECKSIG
        let mut script = vec![0x76, 0xa9, 0x14]; // OP_DUP OP_HASH160 PUSH(20)
        script.extend_from_slice(pubkey_hash);
        script.push(0x88); // OP_EQUALVERIFY
        script.push(0xac); // OP_CHECKSIG

        Ok(script)
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

    /// Convert P2WPKH address to scriptPubKey
    fn p2wpkh_address_to_script(address: &str) -> Result<Vec<u8>, ChainGuardError> {
        use bech32::Variant;

        let (_hrp, data, variant) = bech32::decode(address).map_err(|e| {
            ChainGuardError::InvalidInput {
                msg: format!("Bech32 decode failed: {}", e),
            }
        })?;

        if variant != Variant::Bech32 {
            return Err(ChainGuardError::InvalidInput {
                msg: "Invalid Bech32 variant for P2WPKH".to_string(),
            });
        }

        // First element is witness version, rest is witness program
        if data.is_empty() {
            return Err(ChainGuardError::InvalidInput {
                msg: "Empty Bech32 data".to_string(),
            });
        }

        let _witness_version = data[0].to_u8();

        // Convert the witness program from base32 (5-bit) to bytes (8-bit)
        let data_u8: Vec<u8> = data[1..].iter().map(|u5| u5.to_u8()).collect();
        let witness_program = Self::convert_bits(&data_u8, 5, 8, false)?;

        // Validate witness program length for P2WPKH (should be 20 bytes)
        if witness_program.len() != 20 {
            return Err(ChainGuardError::InvalidInput {
                msg: format!("Invalid P2WPKH witness program length: {} (expected 20)", witness_program.len()),
            });
        }

        // P2WPKH scriptPubKey: OP_0 <20-byte-hash>
        let mut script = vec![0x00, 0x14]; // OP_0 PUSH(20)
        script.extend_from_slice(&witness_program);

        Ok(script)
    }

    /// Convert P2TR address to scriptPubKey
    fn p2tr_address_to_script(address: &str) -> Result<Vec<u8>, ChainGuardError> {
        use bech32::Variant;

        let (_hrp, data, variant) = bech32::decode(address).map_err(|e| {
            ChainGuardError::InvalidInput {
                msg: format!("Bech32m decode failed: {}", e),
            }
        })?;

        if variant != Variant::Bech32m {
            return Err(ChainGuardError::InvalidInput {
                msg: "Invalid Bech32m variant for P2TR".to_string(),
            });
        }

        // First element is witness version, rest is witness program
        if data.is_empty() {
            return Err(ChainGuardError::InvalidInput {
                msg: "Empty Bech32m data".to_string(),
            });
        }

        let _witness_version = data[0].to_u8();

        // Convert the witness program from base32 (5-bit) to bytes (8-bit)
        let data_u8: Vec<u8> = data[1..].iter().map(|u5| u5.to_u8()).collect();
        let witness_program = Self::convert_bits(&data_u8, 5, 8, false)?;

        // Validate witness program length for P2TR (should be 32 bytes)
        if witness_program.len() != 32 {
            return Err(ChainGuardError::InvalidInput {
                msg: format!("Invalid P2TR witness program length: {} (expected 32)", witness_program.len()),
            });
        }

        // P2TR scriptPubKey: OP_1 <32-byte-x-only-pubkey>
        let mut script = vec![0x51, 0x20]; // OP_1 PUSH(32)
        script.extend_from_slice(&witness_program);

        Ok(script)
    }

    /// Base58 decode
    fn base58_decode(encoded: &str) -> Result<Vec<u8>, ChainGuardError> {
        const ALPHABET: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

        let mut num = num_bigint::BigUint::from(0u32);
        let base = num_bigint::BigUint::from(58u32);

        for c in encoded.chars() {
            let digit = ALPHABET
                .find(c)
                .ok_or_else(|| ChainGuardError::InvalidInput {
                    msg: format!("Invalid Base58 character: {}", c),
                })?;

            num = num * &base + digit;
        }

        let mut decoded = num.to_bytes_be();

        // Add leading zeros
        for c in encoded.chars() {
            if c == '1' {
                decoded.insert(0, 0);
            } else {
                break;
            }
        }

        Ok(decoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_public_key() {
        // Example uncompressed public key (65 bytes)
        let uncompressed = vec![
            0x04, // Prefix
            // X coordinate (32 bytes)
            0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62, 0x95, 0xce, 0x87,
            0x0b, 0x07, 0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9, 0x59, 0xf2, 0x81, 0x5b,
            0x16, 0xf8, 0x17, 0x98,
            // Y coordinate (32 bytes)
            0x48, 0x3a, 0xda, 0x77, 0x26, 0xa3, 0xc4, 0x65, 0x5d, 0xa4, 0xfb, 0xfc, 0x0e, 0x11,
            0x08, 0xa8, 0xfd, 0x17, 0xb4, 0x48, 0xa6, 0x85, 0x54, 0x19, 0x9c, 0x47, 0xd0, 0x8f,
            0xfb, 0x10, 0xd4, 0xb8,
        ];

        let compressed = BitcoinAddress::compress_public_key(&uncompressed).unwrap();

        assert_eq!(compressed.len(), 33);
        // Y coordinate is even (last byte 0xb8), so prefix should be 0x02
        assert_eq!(compressed[0], 0x02);
    }

    #[test]
    fn test_base58_encode_decode() {
        let data = vec![0x00, 0x01, 0x02, 0x03, 0x04];
        let encoded = BitcoinAddress::base58_encode(&data);
        let decoded = BitcoinAddress::base58_decode(&encoded).unwrap();

        assert_eq!(data, decoded);
    }
}
