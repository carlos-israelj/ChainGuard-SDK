# ChainGuard SDK

**Security middleware for AI agents on ICP**

ChainGuard SDK is a security-focused middleware that enables AI agents to execute multi-chain transactions with granular access control, threshold signatures, and complete auditability.

## Features

- **Access Control**: Role-based permissions (Owner, Operator, Viewer)
- **Threshold Signatures**: N-of-M approval system using ICP Chain-Key
- **Policy Engine**: Configurable policies with conditions (max amount, daily limits, whitelists, etc.)
- **Multi-Chain Support**: Ethereum/EVM chains (via EVM RPC Canister), Bitcoin (planned)
- **Audit Trail**: Complete on-chain logging of all actions
- **Emergency Controls**: Pause/resume functionality
- **Chain-Key ECDSA**: Threshold signature signing for Ethereum transactions

## Architecture

```
AI Agent → ChainGuard Canister → Multi-Chain Executor
                ↓
         Access Control → Policy Evaluation → Threshold Signing → Execution → Audit Log
```

## Project Structure

```
chainguard-sdk/
├── src/chainguard/
│   ├── src/
│   │   ├── lib.rs              # Main entry point
│   │   ├── types.rs            # Type definitions
│   │   ├── access_control.rs   # RBAC & policies
│   │   ├── threshold.rs        # Multi-sig logic
│   │   ├── audit.rs            # Audit logging
│   │   ├── errors.rs           # Error types
│   │   ├── executor.rs         # Multi-chain executor
│   │   ├── evm_rpc.rs          # EVM RPC integration
│   │   ├── config.rs           # Configuration (git ignored)
│   │   └── config.example.rs   # Configuration template
│   ├── chainguard.did          # Candid interface
│   └── Cargo.toml
├── dfx.json
├── Cargo.toml
├── canister_ids.json
└── README.md
```

## Prerequisites

- Rust (latest stable)
- dfx (ICP SDK)
- wasm32-unknown-unknown target

## Installation

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dfx
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"

# Add wasm target
rustup target add wasm32-unknown-unknown

# Clone and build
git clone https://github.com/carlos-israelj/ChainGuard-SDK.git
cd ChainGuard-SDK
cargo build --target wasm32-unknown-unknown --release
```

## Configuration

### Alchemy API Key Setup

The project requires an Alchemy API key for Sepolia RPC access.

1. Copy the config template:
```bash
cp src/chainguard/src/config.example.rs src/chainguard/src/config.rs
```

2. Edit `src/chainguard/src/config.rs` and replace `YOUR_ALCHEMY_API_KEY` with your actual Alchemy API key:
```rust
pub const ALCHEMY_API_KEY: &str = "your_actual_api_key_here";
```

**Security Note:** The `config.rs` file is git-ignored. Never commit it to version control.

## Quick Start

### 1. Start Local Replica

```bash
export PATH="$HOME/.local/share/dfx/bin:$PATH"
dfx start --background
```

### 2. Deploy Canister

```bash
dfx deploy chainguard
```

### 3. Initialize with Configuration

```bash
dfx canister call chainguard initialize '(
  record {
    name = "My ChainGuard Instance";
    default_threshold = record { required = 2; total = 3 };
    supported_chains = vec { "Sepolia"; "Ethereum" };
    policies = vec {
      record {
        name = "Small Transactions";
        conditions = vec { variant { MaxAmount = 1000000000000000000 } };
        action = variant { Allow };
        priority = 1
      }
    }
  }
)' --network ic
```

### 4. Assign Roles

```bash
# Assign Operator role
dfx canister call chainguard assign_role '(
  principal "xxxxx-xxxxx-xxxxx-xxxxx-xxx",
  variant { Operator }
)'
```

### 5. Request an Action

```bash
dfx canister call chainguard request_action '(
  variant {
    Transfer = record {
      chain = "Sepolia";
      token = "ETH";
      to = "0x648a3e5510f55B4995fA5A22cCD62e2586ACb901";
      amount = 1000000000000000
    }
  }
)' --network ic
```

## API Reference

### Role Management

- `assign_role(principal, role)` - Assign a role to a principal
- `revoke_role(principal, role)` - Revoke a role
- `get_roles(principal)` - Get roles for a principal
- `list_role_assignments()` - List all role assignments

### Policy Management

- `add_policy(policy)` - Add a new policy
- `update_policy(id, policy)` - Update existing policy
- `remove_policy(id)` - Remove a policy
- `list_policies()` - List all policies

### Action Execution

- `request_action(action)` - Request an action (swap, transfer, approve)
- Returns: `Executed`, `PendingSignatures`, or `Denied`

### Threshold Signing

- `get_pending_requests()` - Get all pending requests
- `sign_request(id)` - Sign a pending request
- `reject_request(id, reason)` - Reject a request

### Audit

- `get_audit_logs(start, end)` - Get audit entries within time range
- `get_audit_entry(id)` - Get specific audit entry

### Emergency

- `pause()` - Pause all operations
- `resume()` - Resume operations
- `is_paused()` - Check if system is paused

## Development Status

### Completed Features ✅
- [x] Role-based access control (Owner, Operator, Viewer)
- [x] Policy engine with configurable conditions
- [x] Threshold signature system
- [x] Audit logging
- [x] Emergency pause/resume controls
- [x] EVM RPC integration with Chain-Key ECDSA
- [x] EIP-1559 transaction signing
- [x] Real Ethereum transaction execution (tested on Sepolia)
- [x] Automatic fee estimation
- [x] Deterministic address derivation

### In Progress 🚧
- [ ] Integration tests
- [ ] Policy evaluation test suite
- [ ] Threshold signing flow tests

### Planned Features 📋
- [ ] Bitcoin integration
- [ ] Multi-sig wallet support
- [ ] Advanced policy conditions
- [ ] Demo AI agent implementation
- [ ] Comprehensive documentation

## Testing

```bash
# Run unit tests
cargo test

# Build for production
cargo build --target wasm32-unknown-unknown --release

# Check canister info
dfx canister call chainguard get_config
```

## Hackathon Submission

- **Bounty**: Secure Multi-Chain AI Agents with ICP (Zo House Hackathon)
- **Prize**: 500 USDC / 63 ICP tokens
- **Deadline**: 27 days from project start

## License

MIT

## Contributing

Contributions are welcome! Please open an issue or PR.

## Resources

- [ICP Rust CDK](https://docs.rs/ic-cdk/latest/ic_cdk/)
- [EVM RPC Canister](https://github.com/internet-computer-protocol/evm-rpc-canister)
- [ICP Threshold ECDSA](https://internetcomputer.org/docs/building-apps/network-features/signatures/t-ecdsa)
- [Alchemy Sepolia Faucet](https://www.alchemy.com/faucets/ethereum-sepolia)
