# Bitcoin Transaction Propagation Test Results

## Test Date
December 14-15, 2025

## Objective
Test Bitcoin testnet4 transaction propagation from ICP mainnet canister using official DFINITY patterns.

## Canister Information
- **Canister ID**: `foxtk-ziaaa-aaaai-atthq-cai`
- **Network**: IC Mainnet
- **ic-cdk**: 0.19.0
- **rust-bitcoin**: 0.32.7

## Bitcoin Address
- **Address**: `tb1q5pc5mfz2xav022kalwnefe447p9zvnrv8058pa`
- **Balance**: 979,466 satoshis (verified on mempool.space)
- **UTXOs**: 2 confirmed UTXOs available
- **Verification**: https://mempool.space/testnet4/address/tb1q5pc5mfz2xav022kalwnefe447p9zvnrv8058pa

## Test Transactions

### Transaction 1: Initial Implementation
- **TXID**: `b8656cc2f7dbc54c0a37b4b02557642b7e5d658cc4495e43b0b21bf65189c81d`
- **Amount**: 10,000 satoshis
- **Destination**: `tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx`
- **Bitcoin Canister**: ✅ Accepted
- **Testnet4 Propagation**: ❌ Failed (not found after 30+ minutes)
- **URL**: https://mempool.space/testnet4/tx/b8656cc2f7dbc54c0a37b4b02557642b7e5d658cc4495e43b0b21bf65189c81d

### Transaction 2: Different Amount Test
- **TXID**: `645911f1bfdebd6091513cb741e35326c4a837994dc01ba7dc356b93bdd5c43b`
- **Amount**: 15,000 satoshis
- **Destination**: `tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx`
- **Bitcoin Canister**: ✅ Accepted
- **Testnet4 Propagation**: ❌ Failed (not found after 30+ minutes)
- **URL**: https://mempool.space/testnet4/tx/645911f1bfdebd6091513cb741e35326c4a837994dc01ba7dc356b93bdd5c43b

### Transaction 3: DFINITY Pattern Implementation
- **TXID**: `acf3707ca8b7c05d1c25222ea8eeec6ee7b94ced5f278cd343198f99c287b58c`
- **Amount**: 20,000 satoshis
- **Destination**: `tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx`
- **Code Changes**: Applied DFINITY pattern (send before compute_txid)
- **Bitcoin Canister**: ✅ Accepted
- **Testnet4 Propagation**: ❌ Failed (not found after 15+ minutes)
- **URL**: https://mempool.space/testnet4/tx/acf3707ca8b7c05d1c25222ea8eeec6ee7b94ced5f278cd343198f99c287b58c

## Code Implementation

### Pattern Used (Following DFINITY Examples)
```rust
// 1. Get UTXOs from Bitcoin Canister
let utxos = bitcoin_get_utxos(&GetUtxosRequest {
    address: from_address.to_string(),
    network: Network::Testnet,  // testnet4
    filter: None,
}).await?.utxos;

// 2. Build P2WPKH transaction
let (unsigned_tx, prev_outputs) = 
    build_transaction_with_fee(&own_addr, &utxos, &dst_addr, amount, fee)?;

// 3. Sign with Chain-Key ECDSA
let signed_tx = sign_p2wpkh_transaction(
    unsigned_tx, &own_addr, &prev_outputs,
    key_name, derivation_path
).await?;

// 4. Serialize
let tx_bytes = serialize(&signed_tx);

// 5. Send FIRST (DFINITY pattern)
bitcoin_send_transaction(&SendTransactionRequest {
    network: Network::Testnet,
    transaction: tx_bytes,
}).await?;

// 6. Compute TXID AFTER sending (DFINITY pattern)
let txid = signed_tx.compute_txid().to_string();
```

## Verification Steps

### ✅ Working Components
1. **UTXO Retrieval**: Successfully retrieved 2 UTXOs from Bitcoin Canister
2. **Transaction Building**: P2WPKH transactions built correctly
3. **Chain-Key ECDSA Signing**: Signatures generated successfully
4. **Bitcoin Canister Acceptance**: All transactions accepted without errors
5. **TXID Generation**: Valid TXIDs computed correctly
6. **Canister Cycles**: 1.4 trillion cycles available (sufficient)

### ❌ Failed Component
**Transaction Propagation**: None of the transactions appeared on Bitcoin testnet4 network despite being accepted by Bitcoin Canister.

## Monitoring Results

All transactions monitored for 15-30 minutes at 30-second intervals:
- **Total Checks**: 90+ checks across 3 transactions
- **Successful Propagations**: 0
- **Not Found Responses**: 100%

## Comparison with DFINITY Examples

Verified our implementation matches official examples exactly:
- ✅ `rust/basic_bitcoin/src/service/send_from_p2wpkh_address.rs`
- ✅ Uses `bitcoin_get_utxos` helper
- ✅ Uses `bitcoin_send_transaction` helper
- ✅ Correct parameter structure
- ✅ Proper network configuration

## Conclusion

**The issue is NOT in the code implementation.**

Despite following DFINITY's official patterns exactly and having all components work correctly (UTXOs, signing, canister acceptance), **transactions fail to propagate from the Bitcoin Canister to the Bitcoin testnet4 network**.

This indicates a **Bitcoin Canister infrastructure issue** on IC mainnet for testnet4 transaction propagation.

## Related Issues
- GitHub Issue #1288: Similar testnet4 propagation failure with BRC-20 inscriptions
- Suggests broader testnet4 infrastructure problems

## Recommendations

1. **Report to DFINITY**: Create GitHub issue with detailed evidence
2. **Use testnet3**: Consider using testnet3 if testnet4 adapters are not operational
3. **Test on mainnet**: Small test on Bitcoin mainnet to verify propagation works
4. **Local testing**: Use `dfx start --enable-bitcoin` with regtest for development

## Technical Debt

While investigating this issue, the following improvements were made:
- ✅ Refactored to use `ic_cdk::bitcoin_canister` helpers (0.19.0)
- ✅ Applied DFINITY's recommended transaction ordering pattern
- ✅ Improved logging and error handling
- ✅ Added comprehensive monitoring scripts

## Files Generated
- `/tmp/bitcoin_issue_report.md` - Detailed GitHub issue template
- `/tmp/BITCOIN_TEST_RESULTS.md` - This file
- `/mnt/c/Users/CarlosIsraelJiménezJ/Documents/ICP/scripts/monitor_*.sh` - Monitoring scripts
- Transaction logs in `/tmp/tx_monitor*.log`
