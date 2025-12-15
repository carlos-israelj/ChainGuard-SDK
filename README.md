# 🛡️ ChainGuard SDK

> **Security middleware for autonomous AI agents on Internet Computer Protocol**

ChainGuard SDK enables AI agents to execute multi-chain transactions with enterprise-grade security through role-based access control, threshold signatures, policy enforcement, and complete auditability—all powered by ICP's Chain-Key cryptography.

[![npm version](https://img.shields.io/npm/v/@chainguarsdk/sdk?style=flat-square&logo=npm)](https://www.npmjs.com/package/@chainguarsdk/sdk)
[![Deployed on ICP](https://img.shields.io/badge/Deployed-IC%20Mainnet-3B00B9?style=flat-square&logo=internet-computer)](https://dashboard.internetcomputer.org/canister/foxtk-ziaaa-aaaai-atthq-cai)
[![Tests Passing](https://img.shields.io/badge/Tests-90%2B%20Passing-18C39F?style=flat-square)](./src/chainguard/src)
[![Rust](https://img.shields.io/badge/Rust-1.75+-ED1E79?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](./LICENSE)

---

## 📋 Table of Contents

- [Deployed Contracts](#-deployed-contracts)
- [The Problem](#-the-problem)
- [Our Solution](#-our-solution)
- [Key Features](#-key-features)
- [Quick Start](#-quick-start)
- [Architecture](#-architecture)
- [Use Cases](#-use-cases)
- [Development](#-development)
- [Roadmap](#-roadmap)
- [Resources](#-resources)

---

## 🌐 Deployed Contracts

| Component | Network | Canister/Contract ID | Explorer |
|-----------|---------|---------------------|----------|
| **ChainGuard Canister** | IC Mainnet | `foxtk-ziaaa-aaaai-atthq-cai` | [Dashboard](https://dashboard.internetcomputer.org/canister/foxtk-ziaaa-aaaai-atthq-cai) |
| **ETH Address (Sepolia)** | Sepolia Testnet | `0xfdd0e2016079951225bd88a41b1b7295aa995cad` | [Etherscan](https://sepolia.etherscan.io/address/0xfdd0e2016079951225bd88a41b1b7295aa995cad) |
| **BTC Address (Testnet)** | Bitcoin Testnet | `tb1q5pc5mfz2xav022kalwnefe447p9zvnrv8058pa` | [Blockstream](https://blockstream.info/testnet/address/tb1q5pc5mfz2xav022kalwnefe447p9zvnrv8058pa) |
| **Frontend Dashboard** | Localhost | `http://localhost:3000` | Development |

**Verified Transactions:**
- ✅ **ETH Transfer**: [0xfd8d8b...240ad](https://sepolia.etherscan.io/tx/0xfd8d8b026020e08b06f575702661a76a074c6e34d23f326d84395fec0f9240ad)
- ✅ **USDC→ETH Swap**: [0x5c18f7...f65a9](https://sepolia.etherscan.io/tx/0x5c18f7f6d0bd486d010caa31ba1a0c88bc6d871474d929c6758d224ee72f65a9)
- ✅ **Bitcoin Transfer**: Executed successfully (5,000 satoshis) - [See Test Results](./BITCOIN_TEST_RESULTS.md)

---

## 🎯 The Problem

### For AI Agent Developers
- **Security Risks**: AI agents require access to private keys, creating single points of failure
- **Lack of Oversight**: No granular control over agent actions leads to potential abuse or errors
- **Compliance Gaps**: Difficulty meeting regulatory requirements for automated financial operations
- **Limited Auditability**: No comprehensive logging of AI agent decisions and transactions

### For Institutions & DAOs
- **Trust Issues**: Cannot safely delegate treasury operations to autonomous agents
- **Risk Management**: Need threshold approvals for large transactions without human bottlenecks
- **Regulatory Compliance**: Require complete audit trails and policy enforcement
- **Multi-Chain Complexity**: Managing security across multiple blockchains is operationally complex

### For the DeFi Ecosystem
- **Adoption Barriers**: Security concerns prevent wider adoption of AI-driven DeFi strategies
- **Fragmented Solutions**: Each project implements custom security, leading to inconsistent standards
- **Scalability Issues**: Manual approval processes don't scale with autonomous agent operations

---

## 💡 Our Solution

ChainGuard SDK provides a **decentralized security middleware** that sits between AI agents and blockchain networks, offering:

1. **🔐 Zero Private Key Exposure**: Uses ICP Chain-Key ECDSA for threshold signing—no private keys stored
2. **🎛️ Granular Access Control**: Role-based permissions (Owner, Operator, Viewer) with customizable policies
3. **✅ Threshold Signatures**: N-of-M approval system requiring multiple authorized signers for high-value actions
4. **📊 Complete Auditability**: Immutable on-chain logging of every action with policy evaluation results
5. **⛓️ Multi-Chain Support**: Unified interface for Ethereum, Sepolia, and future EVM/Bitcoin chains
6. **🚨 Emergency Controls**: Pause/resume functionality for instant risk mitigation

---

## 🌟 Key Features

### 🔒 Security Architecture

**Chain-Key ECDSA Integration**
- Deterministic address derivation per canister
- Threshold signature signing without private key exposure
- ICP's `test_key_1` for mainnet testing
- EIP-1559 transaction support with automatic fee estimation

**Role-Based Access Control (RBAC)**
- **Owner**: Full system control, policy management, role assignment
- **Operator**: Execute transactions within policy limits
- **Viewer**: Read-only access to logs and configurations

**Policy Engine**
- ✅ Max/Min amount limits per transaction
- ✅ Daily/hourly spending limits
- ✅ Allowed chains and token whitelists
- ✅ Time-based restrictions (business hours, cooldown periods)
- ✅ Priority-based policy evaluation (first match wins)
- ✅ Default-deny security model

### ⚡ Performance & Scalability

| Metric | Value | Notes |
|--------|-------|-------|
| **Transaction Latency** | ~3-5 seconds | Including policy evaluation + signing |
| **Gas Costs (Sepolia)** | ~21,055 gas | ETH transfers (testnet verified) |
| **Canister Cycles Cost** | ~10B cycles/RPC call | EVM RPC canister integration |
| **Concurrent Strategies** | Unlimited | Limited only by cycles balance |
| **Audit Log Storage** | On-chain | Immutable, queryable by time range |

### 🌐 Multi-Chain Support

| Chain | Status | Features | Address Format |
|-------|--------|----------|----------------|
| **Ethereum Mainnet** | ✅ Ready | Transfers, Swaps (Uniswap V3), ERC20 | 0x... |
| **Sepolia Testnet** | ✅ Verified | Full testing environment | 0x... |
| **Bitcoin Mainnet** | ✅ Ready | Native BTC transfers, UTXO management | bc1... (P2WPKH/P2TR) |
| **Bitcoin Testnet** | ✅ Ready | Full testing with testnet BTC | tb1... (P2WPKH/P2TR) |
| **Other EVM Chains** | 🟡 Extensible | Configurable via RPC endpoints | 0x... |

**Bitcoin Integration Features:**
- ✅ P2PKH (Legacy) address support with manual DER signature encoding
- ✅ P2WPKH (SegWit v0) with BIP143 sighash implementation
- ✅ P2TR (Taproot) address generation (signing limited by ICP ECDSA)
- ✅ UTXO management with greedy selection algorithm
- ✅ Dynamic fee estimation using Bitcoin canister's fee percentiles
- ✅ Change output handling (dust limit: 546 satoshis)
- ✅ Integration with ICP's Bitcoin canister for mainnet/testnet
- ✅ Chain-Key ECDSA signing for Bitcoin transactions

---

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dfx (ICP SDK)
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"

# Add wasm target
rustup target add wasm32-unknown-unknown
```

### Installation & Configuration

```bash
# 1. Clone repository
git clone https://github.com/carlos-israelj/ChainGuard-SDK.git
cd ChainGuard-SDK

# 2. Set up dfx path (WSL/Linux)
export PATH="$HOME/.local/share/dfx/bin:$PATH"

# 3. Configure Alchemy API key (required for RPC)
cp src/chainguard/src/config.example.rs src/chainguard/src/config.rs
# Edit config.rs and add your Alchemy API key

# 4. Build canister
dfx build chainguard

# 5. Deploy to IC mainnet
dfx deploy chainguard --network ic
```

### Initialize ChainGuard

```bash
dfx canister call chainguard initialize '(
  record {
    name = "ChainGuard Production";
    default_threshold = record { required = 2; total = 3 };
    supported_chains = vec { "Sepolia"; "Ethereum" };
    policies = vec {
      record {
        name = "Allow Small Transfers";
        conditions = vec {
          variant { MaxAmount = 1000000000000000000 };
          variant { AllowedChains = vec { "Sepolia" } };
        };
        action = variant { Allow };
        priority = 1;
      };
      record {
        name = "Threshold for Large Transfers";
        conditions = vec {
          variant { MinAmount = 1000000000000000000 };
        };
        action = variant {
          RequireThreshold = record {
            required = 2;
            from_roles = vec { variant { Operator }; variant { Owner } };
          }
        };
        priority = 2;
      }
    }
  }
)' --network ic
```

### Execute Your First Transaction

**Ethereum Transfer:**
```bash
# Request ETH transfer (will be evaluated by policies)
dfx canister call chainguard request_action '(
  variant {
    Transfer = record {
      chain = "Sepolia";
      token = "ETH";
      to = "0x648a3e5510f55B4995fA5A22cCD62e2586ACb901";
      amount = 1000000000000000  # 0.001 ETH
    }
  }
)' --network ic
```

**Bitcoin Transfer:**
```bash
# Get your Bitcoin testnet address
dfx canister call chainguard get_bitcoin_address '("BitcoinTestnet")' --network ic
# Returns: "tb1q..."

# Send Bitcoin (100,000 satoshis = 0.001 BTC)
dfx canister call chainguard request_action '(
  variant {
    BitcoinTransfer = record {
      to = "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx";
      amount = 100000;
      network = "BitcoinTestnet"
    }
  }
)' --network ic
```

### Using the TypeScript SDK

For application development, use the published npm package:

```bash
# Install the SDK
npm install @chainguarsdk/sdk
```

```typescript
import { ChainGuardClient } from '@chainguarsdk/sdk';

// Initialize client
const client = new ChainGuardClient({
  canisterId: 'foxtk-ziaaa-aaaai-atthq-cai',
});

// Ethereum transfer
const ethResult = await client.transfer(
  'Sepolia',
  'ETH',
  '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0',
  BigInt(1000000000000000)  // 0.001 ETH
);

// Bitcoin transfer
const btcResult = await client.bitcoinTransfer(
  'tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx',  // Recipient address
  BigInt(100000),                                   // 0.001 BTC in satoshis
  'BitcoinTestnet'
);

// Get Bitcoin address
const addressResult = await client.getBitcoinAddress('BitcoinTestnet');
if ('Ok' in addressResult) {
  console.log('Bitcoin Address:', addressResult.Ok);
}
```

**See the complete [SDK documentation](./packages/sdk/) for all features and examples.**

---

## 🏗️ Architecture

### System Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                      AI Agent / Frontend                        │
│            (TypeScript Client or Next.js Dashboard)             │
└────────────────────────┬────────────────────────────────────────┘
                         │ @dfinity/agent
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                   ChainGuard Canister (ICP)                     │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐    │
│  │ Access       │→ │ Policy       │→ │ Threshold          │    │
│  │ Control      │  │ Engine       │  │ Signer             │    │
│  │ (RBAC)       │  │              │  │                    │    │
│  └──────────────┘  └──────────────┘  └────────┬───────────┘    │
│                                                │                │
│  ┌──────────────────────────────────────────┐ │                │
│  │        Multi-Chain Executor               │ │                │
│  │  ┌─────────────┐    ┌─────────────┐     │ │                │
│  │  │ EVM RPC     │    │ Bitcoin     │     │ │                │
│  │  │ Executor    │    │ Executor    │     │ │                │
│  │  └─────────────┘    └─────────────┘     │ │                │
│  └──────────────────────────────────────────┘ │                │
│                                                │                │
│  ┌──────────────────────────────────────────┐ │                │
│  │           Audit Log (On-Chain)           │◄┘                │
│  └──────────────────────────────────────────┘                  │
└────────────┬───────────────────────┬────────────────────────────┘
             │                       │
             │ Chain-Key ECDSA       │ EVM RPC Canister
             ▼                       ▼
   ┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐
   │ ICP Threshold    │    │ Ethereum/Sepolia │    │ Bitcoin Network  │
   │ ECDSA Subnet     │    │ (via Alchemy)    │    │ (via BTC Canister│
   └──────────────────┘    └──────────────────┘    └──────────────────┘
```

### Transaction Lifecycle

```
1. Agent Submits Action
        ↓
2. Access Control Check (RBAC)
        ↓
3. Policy Evaluation (conditions, limits)
        ↓
4. Decision Branch:
   ├─→ [Allow] → Execute Immediately
   ├─→ [Deny] → Return Error
   └─→ [RequireThreshold] → Create Pending Request
        ↓
5. Threshold Signing (if required)
   ├─→ Collect N signatures
   └─→ Auto-execute when threshold met
        ↓
6. Multi-Chain Executor
   ├─→ EVM RPC (Ethereum, Sepolia)
   │   ├─→ Get nonce
   │   ├─→ Estimate gas
   │   ├─→ Sign with Chain-Key ECDSA
   │   ├─→ Submit transaction
   │   └─→ Return tx hash
   └─→ Bitcoin (future)
        ↓
7. Audit Log Entry Created
        ↓
8. Return Result to Agent
```

---

## 💼 Use Cases

### 🤖 Autonomous DeFi Strategies

ChainGuard enables users to configure AI agents for automated trading strategies through an intuitive web interface:

**User Workflow:**
1. **Configure** strategies in the frontend dashboard
2. **AI Agent** executes automatically based on schedule
3. **Approve** threshold signatures when required
4. **Monitor** results in audit logs

**Dollar Cost Averaging (DCA)**
- Configure token pairs and purchase amounts via frontend
- AI agent executes purchases at regular intervals
- Policy-controlled amount limits
- Complete execution history visible in dashboard

**Portfolio Rebalancing**
- Set target allocations through web interface
- AI agent maintains target allocations automatically
- Threshold approvals for large rebalances
- Deviation tracking and logging

**Yield Farming**
- Configure farming strategies in dashboard
- Automated staking and reward claiming
- Multi-chain position management
- Risk-controlled execution

### 🏢 DAO Treasury Management

**Multi-Sig Governance**
- Require N-of-M approvals for treasury operations
- Role-based spending limits
- Transparent audit trail for all votes

**Automated Payments**
- Scheduled contributor payments
- Policy-enforced budget limits
- Complete payment history

### 🔐 Enterprise Security

**Compliance & Reporting**
- Immutable audit logs for regulatory requirements
- Time-based transaction restrictions
- Role separation enforcement

**Risk Management**
- Daily/hourly spending limits
- Emergency pause functionality
- Real-time transaction monitoring

---

## 🛠️ Development

### Project Structure

```
chainguard-sdk/
├── src/chainguard/              # Main canister source
│   ├── src/
│   │   ├── lib.rs               # Canister entry points
│   │   ├── access_control.rs    # RBAC & policies (20+ tests)
│   │   ├── threshold.rs         # Multi-sig logic (15 tests)
│   │   ├── audit.rs             # Audit logging (10 tests)
│   │   ├── executor.rs          # Multi-chain executor
│   │   ├── evm_rpc.rs           # EVM RPC integration
│   │   ├── abi.rs               # ERC20/Uniswap ABI (16 tests)
│   │   ├── universal_router.rs  # Uniswap routing (21 tests)
│   │   └── types.rs             # Type definitions
│   └── chainguard.did           # Candid interface
├── packages/sdk/                # TypeScript SDK (npm package)
│   ├── src/
│   │   ├── index.ts             # Exports & public API
│   │   ├── client.ts            # ChainGuardClient class
│   │   ├── types.ts             # TypeScript types
│   │   └── idl.ts               # Candid IDL factory
│   ├── package.json             # @chainguarsdk/sdk
│   └── README.md                # SDK documentation
├── examples/ai-agent/           # AI Agent implementation
│   ├── src/
│   │   ├── strategies/
│   │   │   ├── dca.ts           # Dollar Cost Averaging
│   │   │   └── rebalance.ts     # Portfolio rebalancing
│   │   └── examples/demo.ts     # Complete demo
│   └── README.md                # Agent documentation
├── frontend/                    # Next.js Dashboard
│   ├── app/
│   │   ├── page.tsx             # Main dashboard
│   │   ├── strategies/          # Strategy configuration
│   │   ├── signatures/          # Threshold approvals
│   │   └── audit/               # Audit logs viewer
│   ├── lib/hooks/
│   │   └── useChainGuard.ts     # React integration
│   └── components/
│       └── NavigationLayout.tsx # Main layout
└── CLAUDE.md                    # Comprehensive project guide
```

### Testing Suite

**Unit Tests: 90+ Tests**
- `abi.rs`: ERC20, WETH, Uniswap V2/V3, Permit2 encoding (16 tests)
- `universal_router.rs`: Universal Router command encoding (21 tests)
- `access_control.rs`: Policy evaluation and RBAC (20 tests)
- `threshold.rs`: Multi-signature workflows (15 tests)
- `audit.rs`: Audit log tracking (10 tests)

**Integration Tests: 8 End-to-End Scenarios**
- ✅ Canister initialization and configuration
- ✅ Role assignment and permission checks
- ✅ Policy evaluation (allow/deny/threshold)
- ✅ Threshold signing workflow (2-of-3 multisig)
- ✅ Audit log creation and querying
- ✅ Emergency pause/resume controls
- ✅ Policy priority ordering
- ✅ Multiple condition evaluation

```bash
# Run all tests during build
dfx build chainguard

# Run integration tests (requires PocketIC)
cargo test --test integration_tests

# View canister configuration
dfx canister call chainguard get_config --network ic
```

### Running the Complete Stack

**1. Canister (Backend)**
```bash
cd /path/to/ChainGuard-SDK
dfx start --background
dfx deploy chainguard --network ic
```

**2. Frontend Dashboard**
```bash
cd frontend
npm install
npm run dev
# Visit http://localhost:3000
```

**3. AI Agent**
```bash
cd examples/ai-agent
npm install

# Run comprehensive demo
npm run test

# Execute specific strategies
npm run dca       # Dollar Cost Averaging
npm run rebalance # Portfolio Rebalancing
```

---

## 🗺️ Roadmap

### ✅ Phase 1: Core Infrastructure (Completed)

**Q4 2024**
- ✅ Role-based access control system
- ✅ Policy engine with configurable conditions
- ✅ Threshold signature implementation
- ✅ EVM RPC integration with Chain-Key ECDSA
- ✅ Uniswap V3 swap execution
- ✅ Comprehensive test suite (90+ tests)
- ✅ Sepolia testnet verification

### 🔄 Phase 2: AI Agent & Frontend (Current)

**Q1 2025**
- ✅ TypeScript AI Agent with DCA/Rebalancing strategies
- ✅ Next.js Dashboard with ICP design system
- ✅ Real-time threshold signature monitoring
- ✅ Complete audit trail visualization
- 🟡 Production mainnet deployment
- 🟡 Gas optimization and cost analysis
- 🟡 Advanced strategy examples (yield farming, arbitrage)

### 📋 Phase 3: Expansion & Optimization

**Q2 2025**
- [ ] Bitcoin integration via ICP Bitcoin API
- [ ] Advanced policy conditions (multi-token limits, velocity checks)
- [ ] Multi-canister architecture for scalability
- [ ] Scheduled transaction execution (cron-like)
- [ ] Webhook notifications for events
- [ ] GraphQL API for analytics

**Q3 2025**
- [ ] Additional EVM chains (Polygon, Arbitrum, Optimism)
- [ ] Cross-chain atomic swaps
- [ ] Off-chain signature aggregation
- [ ] Mobile app for threshold approvals
- [ ] Analytics dashboard with strategy performance metrics

### 🌟 Phase 4: Enterprise Features

**Q4 2025**
- [ ] White-label deployment options
- [ ] Compliance reporting templates
- [ ] Advanced analytics and risk scoring
- [ ] Integration with institutional custody solutions
- [ ] Multi-tenant architecture

---

## 📚 API Reference

### Core Canister Methods

**Initialization**
```rust
initialize(config: InitConfig) -> Result<(), String>
```

**Role Management**
```rust
assign_role(principal: Principal, role: Role) -> Result<(), String>
revoke_role(principal: Principal, role: Role) -> Result<(), String>
get_roles(principal: Principal) -> Vec<Role>
list_role_assignments() -> Vec<(Principal, Vec<Role>)>
```

**Policy Management**
```rust
add_policy(policy: Policy) -> Result<u64, String>
update_policy(id: u64, policy: Policy) -> Result<(), String>
remove_policy(id: u64) -> Result<(), String>
list_policies() -> Vec<(u64, Policy)>
```

**Action Execution**
```rust
request_action(action: Action) -> ActionResult
// ActionResult: Executed | PendingSignatures | Denied
```

**Threshold Signatures**
```rust
get_pending_requests() -> Vec<PendingRequest>
sign_request(id: u64) -> Result<ActionResult, String>
reject_request(id: u64, reason: String) -> Result<(), String>
```

**Audit & Monitoring**
```rust
get_audit_logs(start: u64, end: u64) -> Vec<AuditEntry>
get_audit_entry(id: u64) -> Option<AuditEntry>
get_config() -> InitConfig
is_paused() -> bool
```

**Emergency Controls**
```rust
pause() -> Result<(), String>
resume() -> Result<(), String>
```

See [CLAUDE.md](./CLAUDE.md) for complete API documentation and examples.

---

## 📖 Resources

### Documentation
- **[CLAUDE.md](./CLAUDE.md)** - Comprehensive project guide for developers
- **[AI Agent README](./examples/ai-agent/README.md)** - Autonomous strategy implementation guide
- **[Frontend README](./frontend/README.md)** - Dashboard setup and usage

### Internet Computer
- [ICP Developer Documentation](https://internetcomputer.org/docs)
- [Rust CDK Reference](https://docs.rs/ic-cdk/latest/ic_cdk/)
- [Chain-Key ECDSA Guide](https://internetcomputer.org/docs/current/developer-docs/integrations/t-ecdsa/)
- [EVM RPC Canister](https://github.com/internet-computer-protocol/evm-rpc-canister)
- [Developer Forum](https://forum.dfinity.org/)

### DeFi Protocols
- [Uniswap V3 Documentation](https://docs.uniswap.org/protocol/introduction)
- [EIP-1559 Transaction Format](https://eips.ethereum.org/EIPS/eip-1559)
- [Ethereum JSON-RPC](https://ethereum.org/en/developers/docs/apis/json-rpc/)
- [Alchemy Sepolia Faucet](https://www.alchemy.com/faucets/ethereum-sepolia)

### Tools & Libraries
- [@dfinity/agent](https://www.npmjs.com/package/@dfinity/agent) - ICP JavaScript agent
- [@dfinity/candid](https://www.npmjs.com/package/@dfinity/candid) - Candid interface
- [@dfinity/identity](https://www.npmjs.com/package/@dfinity/identity) - Identity management
- [ic-cdk](https://crates.io/crates/ic-cdk) - ICP Rust CDK
- [evm-rpc-canister-types](https://crates.io/crates/evm-rpc-canister-types) - EVM RPC types

---

## 🤝 Contributing

We welcome contributions! Here's how to get started:

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make your changes** and add tests
4. **Run the test suite**: `dfx build chainguard && cargo test`
5. **Commit your changes**: `git commit -m 'Add amazing feature'`
6. **Push to the branch**: `git push origin feature/amazing-feature`
7. **Open a Pull Request**

### Development Guidelines
- Follow Rust best practices and clippy recommendations
- Add unit tests for new functionality
- Update integration tests for end-to-end scenarios
- Document public APIs with rustdoc comments
- Update CLAUDE.md for significant architectural changes

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

---

## 🏆 Acknowledgments

**Built for Zo House Hackathon**
- **Bounty**: Secure Multi-Chain AI Agents with ICP
- **Prize**: 500 USDC / 63 ICP tokens

**Special Thanks**
- Internet Computer Protocol team for Chain-Key ECDSA
- DFINITY Foundation for comprehensive developer tools
- Uniswap Labs for decentralized exchange infrastructure
- Alchemy for reliable RPC infrastructure

---

## 📧 Contact & Support

- **GitHub Issues**: [Report bugs or request features](https://github.com/carlos-israelj/ChainGuard-SDK/issues)
- **ICP Forum**: [Developer discussions](https://forum.dfinity.org/)
- **Documentation**: See [CLAUDE.md](./CLAUDE.md) for detailed guides

---

<div align="center">

**Built with ❤️ on Internet Computer Protocol**

[Documentation](./CLAUDE.md) • [AI Agent](./examples/ai-agent/) • [Dashboard](./frontend/) • [ICP Forum](https://forum.dfinity.org/)

</div>
