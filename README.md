# ChainGuard SDK

**Security middleware for AI agents on ICP**

ChainGuard SDK is a security-focused middleware that enables AI agents to execute multi-chain transactions with granular access control, threshold signatures, and complete auditability.

## Features

- **Access Control**: Role-based permissions (Owner, Operator, Viewer)
- **Threshold Signatures**: N-of-M approval system using ICP Chain-Key
- **Policy Engine**: Configurable policies with conditions (max amount, daily limits, whitelists, etc.)
- **Multi-Chain Support**: Ethereum/EVM chains (via ic-alloy), Bitcoin (planned)
- **Audit Trail**: Complete on-chain logging of all actions
- **Emergency Controls**: Pause/resume functionality

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
│   │   └── errors.rs           # Error types
│   ├── chainguard.did          # Candid interface
│   └── Cargo.toml
├── dfx.json
├── Cargo.toml
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
    supported_chains = vec { "ethereum"; "polygon" };
    policies = vec {
      record {
        name = "Small Transactions";
        conditions = vec { variant { MaxAmount = 1000000000 } };
        action = variant { Allow };
        priority = 1
      }
    }
  }
)'
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
      chain = "ethereum";
      token = "0x...USDC";
      to = "0x...recipient";
      amount = 100000000
    }
  }
)'
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

**Week 1** ✅ (Current)
- [x] Project setup
- [x] Type definitions
- [x] Access control module
- [x] Threshold signer module
- [x] Audit log module
- [x] Main canister implementation
- [x] Compilation successful

**Week 2** (Next Steps)
- [ ] Integration tests
- [ ] Add policy evaluation tests
- [ ] Test threshold signing flow

**Week 3** (Planned)
- [ ] ic-alloy integration for Ethereum
- [ ] Real transaction execution
- [ ] Demo AI agent

**Week 4** (Planned)
- [ ] Documentation & examples
- [ ] Video demo
- [ ] Hackathon submission

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
- [ic-alloy Documentation](https://o7kje-7yaaa-aaaal-qnaua-cai.icp0.io/getting-started.html)
- [ICP Threshold ECDSA](https://internetcomputer.org/docs/building-apps/network-features/signatures/t-ecdsa)
