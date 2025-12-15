# Bitcoin Testnet4 Transaction Propagation Failure from IC Mainnet

## Summary
Bitcoin transactions sent via `bitcoin_send_transaction` from IC mainnet canisters are accepted by the Bitcoin Canister but **fail to propagate to Bitcoin testnet4 network**, despite following DFINITY's official implementation patterns exactly.

## Environment
- **Network**: Internet Computer Mainnet
- **Bitcoin Network**: Testnet4
- **Canister ID**: `foxtk-ziaaa-aaaai-atthq-cai`
- **ic-cdk version**: 0.19.0
- **rust-bitcoin version**: 0.32.7
- **Transaction Type**: P2WPKH (Pay-to-Witness-Public-Key-Hash)

## Problem Description
When executing Bitcoin transfers using the official `ic_cdk::bitcoin_canister` helper functions, transactions are:
1. ✅ Successfully built and signed with Chain-Key ECDSA
2. ✅ Accepted by `bitcoin_send_transaction()` without errors
3. ✅ Return valid TXIDs
4. ❌ **Never appear on Bitcoin testnet4 network** (verified via mempool.space and block explorers)

## Reproduction Steps

### Code Implementation (Following DFINITY Pattern)
```rust
use ic_cdk::bitcoin_canister::{
    bitcoin_get_utxos, bitcoin_send_transaction,
    GetUtxosRequest, SendTransactionRequest, Network
};

// 1. Get UTXOs
let utxos = bitcoin_get_utxos(&GetUtxosRequest {
    address: from_address.to_string(),
    network: Network::Testnet,  // Maps to testnet4
    filter: None,
}).await?.utxos;

// 2. Build P2WPKH transaction using rust-bitcoin
let signed_tx = /* ... transaction building and signing ... */;

// 3. Send transaction (DFINITY pattern: send BEFORE computing txid)
bitcoin_send_transaction(&SendTransactionRequest {
    network: Network::Testnet,
    transaction: serialize(&signed_tx),
}).await?;

// 4. Compute txid
let txid = signed_tx.compute_txid().to_string();
```

### Test Execution
```bash
dfx canister call foxtk-ziaaa-aaaai-atthq-cai request_action '(
  variant {
    Transfer = record {
      chain = "BitcoinTestnet";
      token = "BTC";
      to = "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx";
      amount = 20000
    }
  }
)' --network ic
```

### Result
```
variant {
  Executed = record {
    chain = "BitcoinTestnet";
    success = true;
    tx_hash = opt "acf3707ca8b7c05d1c25222ea8eeec6ee7b94ced5f278cd343198f99c287b58c";
  }
}
```

## Evidence of Non-Propagation

### Failed Transaction Examples
All transactions were accepted by Bitcoin Canister but never appeared on testnet4:

1. **TXID**: `b8656cc2f7dbc54c0a37b4b02557642b7e5d658cc4495e43b0b21bf65189c81d`
   - URL: https://mempool.space/testnet4/tx/b8656cc2f7dbc54c0a37b4b02557642b7e5d658cc4495e43b0b21bf65189c81d
   - Status: "Transaction not found" after 30+ minutes

2. **TXID**: `645911f1bfdebd6091513cb741e35326c4a837994dc01ba7dc356b93bdd5c43b`
   - URL: https://mempool.space/testnet4/tx/645911f1bfdebd6091513cb741e35326c4a837994dc01ba7dc356b93bdd5c43b
   - Status: "Transaction not found" after 30+ minutes

3. **TXID**: `acf3707ca8b7c05d1c25222ea8eeec6ee7b94ced5f278cd343198f99c287b58c` (after applying DFINITY pattern fixes)
   - URL: https://mempool.space/testnet4/tx/acf3707ca8b7c05d1c25222ea8eeec6ee7b94ced5f278cd343198f99c287b58c
   - Status: "Transaction not found" after 15+ minutes

### Verified Working Elements
- ✅ **Source address has funds**: `tb1q5pc5mfz2xav022kalwnefe447p9zvnrv8058pa` with ~979,466 satoshis
  - Verified: https://mempool.space/testnet4/address/tb1q5pc5mfz2xav022kalwnefe447p9zvnrv8058pa
- ✅ **UTXOs retrieved successfully** from Bitcoin Canister
- ✅ **Transactions properly signed** with Chain-Key ECDSA
- ✅ **Bitcoin Canister accepts transactions** without rejection errors

## Investigation Performed

### 1. Code Pattern Verification
Compared our implementation line-by-line with DFINITY's official examples:
- ✅ `rust/basic_bitcoin/src/service/send_from_p2wpkh_address.rs`
- ✅ Matches UTXO retrieval pattern
- ✅ Matches transaction building pattern
- ✅ Matches signing pattern
- ✅ Matches `bitcoin_send_transaction` usage

### 2. Order of Operations Fix
Changed from:
```rust
let txid = signed_tx.compute_txid();  // Before
self.send_transaction(tx_bytes).await?;
```

To DFINITY pattern:
```rust
self.send_transaction(tx_bytes).await?;  // Send first
let txid = signed_tx.compute_txid();     // Compute after
```

**Result**: No change in propagation behavior

### 3. Canister Cycles
Verified canister has sufficient cycles: **1.4 trillion cycles**

### 4. Network Configuration
- Using `ic_cdk::bitcoin_canister::Network::Testnet` (maps to testnet4 per ICP docs)
- rust-bitcoin `Network::Testnet` used only for address validation (compatible with testnet4)

## Related Issues
- Issue #1288: Similar testnet4 propagation problem with BRC-20 inscriptions
  - https://github.com/dfinity/examples/issues/1288
  - Suggests broader testnet4 infrastructure issues

## Expected Behavior
Transactions should:
1. Be accepted by Bitcoin Canister
2. Be propagated to Bitcoin testnet4 network via Bitcoin adapters
3. Appear in mempool within 1-5 minutes
4. Be mineable and confirmable

## Actual Behavior
Transactions are accepted but never propagate, remaining invisible to:
- mempool.space testnet4 explorer
- Other testnet4 block explorers
- Bitcoin testnet4 nodes

## Questions for DFINITY Team

1. **Are Bitcoin adapters operational for testnet4 on IC mainnet?**
2. **Is there a known issue with testnet4 transaction propagation?**
3. **Should we use testnet3 instead, or is testnet4 fully supported?**
4. **Are there additional configuration requirements not documented in examples?**
5. **Is there a way to verify transaction status within the Bitcoin Canister queue?**

## Workaround Attempts

Tried:
- ❌ Different transaction amounts (10k, 15k, 20k satoshis)
- ❌ Following DFINITY pattern exactly
- ❌ Reinstalling canister with fresh state
- ❌ Waiting 30+ minutes for asynchronous propagation

None resolved the issue.

## Request
Please investigate Bitcoin Canister's testnet4 transaction propagation on IC mainnet. This appears to be an infrastructure issue rather than implementation issue, as the code matches official examples exactly.

## Repository
ChainGuard SDK - Multi-chain security middleware for ICP
- Uses official `ic-cdk` 0.19.0 Bitcoin integration
- Full source available if needed for debugging
