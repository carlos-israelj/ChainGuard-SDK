# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-12-15

### Changed
- Updated to support ChainGuard canister with stable memory implementation
- Canister state now persists across upgrades automatically
- Package name corrected in README from `@chainguard/sdk` to `@chainguarsdk/sdk`

### Internal
- Canister upgrade mechanism now uses `pre_upgrade` and `post_upgrade` hooks
- All state (config, roles, policies, pending requests, audit entries) persists across canister upgrades
- Uses `ic-stable-structures` v0.6 for type-safe persistence

### Verified
- Tested on IC mainnet (canister: `foxtk-ziaaa-aaaai-atthq-cai`)
- State persistence verified with 2 policies, 2 role assignments
- Compatible with existing deployed canister

## [0.1.0] - 2024-12-14

### Added
- Initial release of ChainGuard TypeScript SDK
- `ChainGuardClient` class with full canister integration
- Helper methods: `transfer()`, `swap()`, `approveToken()`
- Complete type definitions for all Candid interfaces
- Support for Ethereum and Sepolia networks
- Role-based access control types
- Policy management interfaces
- Threshold signature workflow support
- Audit log querying

### Features
- Auto-generated identity support
- Custom identity injection
- Full TypeScript type safety
- ESM and CommonJS support
- Complete API coverage of ChainGuard canister

### Verified Transactions
- ETH Transfer: `0xfd8d8b026020e08b06f575702661a76a074c6e34d23f326d84395fec0f9240ad`
- ETH→USDC Swap: `0x9c30a38f4e0f58bc1dd29c34c5e3f7c31d8dc3f7bab8d31dc0e3ec5eae0f4db9`
- USDC→ETH Swap: `0xbfbdab70dd24fcb72c70b60f94096c67ca5cf949e3e99d201ba088377ed8652a`
