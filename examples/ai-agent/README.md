# ChainGuard AI Agent

Autonomous AI agent for secure multi-chain DeFi strategies on Internet Computer Protocol. Execute automated Dollar Cost Averaging and portfolio rebalancing with policy-based controls, threshold signatures, and complete auditability.

## Overview

The ChainGuard AI Agent demonstrates how to build autonomous trading strategies that interact with the ChainGuard SDK canister. It showcases policy-based access control, threshold signature workflows, and multi-chain transaction execution with Chain-Key ECDSA.

## Features

**Automated Strategies**
- Dollar Cost Averaging (DCA) - Regular purchases at fixed intervals
- Portfolio Rebalancing - Maintain target asset allocations
- Extensible architecture for custom strategies

**Security Architecture**
- Policy-based execution control at canister level
- Threshold signature requirements for large transactions
- Immutable audit trail for all operations
- Role-based access control (Owner, Operator, Viewer)

**Multi-Chain Support**
- Ethereum mainnet via EVM RPC canister
- Sepolia testnet for development
- Extensible to any EVM-compatible chain

**Monitoring & Analytics**
- Real-time strategy execution logs
- Audit trail inspection and filtering
- Pending threshold request monitoring

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    ChainGuard AI Agent                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌────────────────┐  ┌────────────────┐                    │
│  │  DCA Strategy  │  │   Rebalance    │  ... more          │
│  │                │  │   Strategy     │  strategies        │
│  └────────┬───────┘  └───────┬────────┘                    │
│           │                  │                              │
│           └──────────┬───────┘                              │
│                      │                                      │
│           ┌──────────▼───────────┐                          │
│           │  ChainGuard Client   │                          │
│           └──────────┬───────────┘                          │
│                      │                                      │
└──────────────────────┼──────────────────────────────────────┘
                       │
                       │ @dfinity/agent
                       │
           ┌───────────▼────────────┐
           │  ChainGuard Canister   │
           │  (ICP Smart Contract)  │
           └───────────┬────────────┘
                       │
            ┌──────────┴──────────┐
            │                     │
    ┌───────▼────────┐   ┌────────▼────────┐
    │ Chain-Key ECDSA│   │   EVM RPC       │
    │  (Threshold    │   │   Canister      │
    │   Signing)     │   │                 │
    └───────┬────────┘   └────────┬────────┘
            │                     │
            └──────────┬──────────┘
                       │
              ┌────────▼─────────┐
              │  Ethereum/Sepolia│
              │   (Uniswap V3)   │
              └──────────────────┘
```

## Installation

### Prerequisites

- Node.js 18+ and npm
- TypeScript
- Access to ICP network
- ChainGuard canister deployed

### Setup

1. Clone the repository and navigate to the agent directory:

```bash
cd examples/ai-agent
```

2. Install dependencies:

```bash
npm install
```

3. Configure the agent:

```bash
# Copy example config
cp config.example.yaml config.yaml

# Copy environment file
cp .env.example .env

# Edit config.yaml with your settings:
# - Canister ID
# - Network settings
# - Token addresses
# - Strategy parameters
```

4. Set up your identity:

```bash
# Option 1: Use your dfx identity
dfx identity export default > identity.pem

# Option 2: Generate a new identity
# The agent will auto-generate one if not found
```

## Configuration

### config.yaml Structure

```yaml
canister:
  id: "foxtk-ziaaa-aaaai-atthq-cai"  # Your canister ID
  network: "ic"  # "ic" or "local"
  host: "https://icp-api.io"

identity:
  type: "pem"
  path: "./identity.pem"

chains:
  Sepolia:
    tokens:
      USDC:
        address: "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"
        decimals: 6

strategies:
  dca:
    enabled: true
    interval: "0 0 * * *"  # Daily at midnight
    params:
      sourceToken: "USDC"
      targetToken: "ETH"
      amountPerPurchase: "1000000"  # 1 USDC
      chain: "Sepolia"
```

See `config.example.yaml` for complete configuration options.

## Quick Start

### Run Complete Demo

Execute all features in a comprehensive demonstration:

```bash
npm run test
```

This runs `src/examples/demo.ts` which demonstrates:
- ChainGuard canister connection
- Policy-based transaction execution
- Threshold signature workflow
- DCA strategy execution
- Portfolio rebalancing
- Audit log inspection

### Execute Individual Strategies

```bash
# Dollar Cost Averaging - one-time execution
npm run dca

# Portfolio Rebalancing - one-time execution
npm run rebalance
```

### Development Mode

```bash
# Compile TypeScript and run main entry point
npm run dev

# Or build and run compiled code
npm run build
npm start
```

## Strategies

### Dollar Cost Averaging (DCA)

Automatically purchases a target asset at regular intervals with a fixed amount, regardless of market price. This strategy reduces volatility impact and removes emotional decision-making from investing.

**How It Works:**
1. Agent initiates swap from source token to target token
2. ChainGuard evaluates policies (amount limits, chain restrictions)
3. Transaction executes via EVM RPC canister
4. Result logged to audit trail

**Configuration Example:**
```yaml
dca:
  enabled: true
  interval: "0 0 * * *"  # Daily at midnight (cron format)
  params:
    sourceToken: "USDC"
    targetToken: "ETH"
    amountPerPurchase: "1000000"  # 1 USDC (6 decimals)
    chain: "Sepolia"
```

**Benefits:**
- Reduces market timing risk
- Automates consistent accumulation
- Removes emotional trading decisions
- Works across market cycles

### Portfolio Rebalancing

Maintains target asset allocations by automatically executing swaps when portfolio deviations exceed a configured threshold. Ensures disciplined risk management.

**How It Works:**
1. Agent queries canister for current token balances
2. Calculates deviations from target allocations
3. If threshold exceeded, generates rebalancing swaps
4. Executes swaps subject to policy evaluation
5. Logs complete rebalancing actions

**Configuration Example:**
```yaml
rebalance:
  enabled: true
  interval: "0 0 * * 0"  # Weekly on Sunday
  params:
    chain: "Sepolia"
    rebalanceThreshold: 5  # Rebalance if deviation > 5%
    portfolio:
      - token: "ETH"
        targetPercentage: 50
      - token: "USDC"
        targetPercentage: 30
      - token: "WETH"
        targetPercentage: 20
```

**Benefits:**
- Maintains target risk profile
- Automatic profit-taking from outperformers
- Buys underperformers at lower prices
- Disciplined, rule-based execution

## Policy Engine Integration

All agent transactions are evaluated by the ChainGuard policy engine before execution. Policies define rules based on amount limits, allowed chains, daily limits, and role requirements.

### Policy Evaluation Flow

1. Agent calls `request_action()` with transaction details
2. ChainGuard evaluates policies in priority order (lowest first)
3. First matching policy determines outcome:
   - `Allow` - Execute immediately
   - `Deny` - Reject transaction
   - `RequireThreshold` - Create pending request for multi-sig approval
4. If no policies match, action is denied by default

### Example Policy Configuration

**Auto-approve small DCA purchases:**
```typescript
{
  name: "Allow Small DCA Swaps",
  conditions: [
    { MaxAmount: 10000000 },  // Max 10 USDC
    { AllowedChains: ["Sepolia"] },
    { DailyLimit: 100000000 }  // 100 USDC per day
  ],
  action: { Allow: null },
  priority: 1
}
```

**Require threshold signatures for large rebalances:**
```typescript
{
  name: "Threshold for Large Rebalances",
  conditions: [
    { MinAmount: 100000000 }  // > 100 USDC
  ],
  action: {
    RequireThreshold: {
      required: 2,
      from_roles: ["Operator", "Owner"]
    }
  },
  priority: 2
}
```

### Setting Policies via CLI

```bash
# View current policies
dfx canister call chainguard list_policies --network ic

# Add a new policy (during initialization)
dfx canister call chainguard initialize '(
  record {
    name = "ChainGuard Instance";
    policies = vec {
      record {
        name = "Allow Agent DCA";
        conditions = vec {
          variant { MaxAmount = 10000000 };
          variant { AllowedChains = vec { "Sepolia" } };
        };
        action = variant { Allow };
        priority = 1;
      }
    }
  }
)' --network ic
```

## Threshold Signature Workflow

When a policy requires multi-signature approval, the transaction enters a pending state until the required number of authorized signers approve it.

### Workflow Steps

1. **Agent submits action** - Calls `request_action()` with transaction details
2. **Policy evaluation** - ChainGuard checks if action matches threshold policy
3. **Pending request created** - Transaction stored with unique ID, awaits signatures
4. **Signers approve** - Authorized principals call `sign_request(id)`
5. **Threshold met** - When required signatures collected, transaction auto-executes
6. **Audit logging** - Complete workflow recorded with timestamps and signers

### Monitoring Pending Requests

**Via Agent:**
```typescript
import { ChainGuardClient } from './utils/chainguard-client';

const client = new ChainGuardClient(canisterId, agent);

// Get all pending requests
const pending = await client.getPendingRequests();
console.log(`Pending requests: ${pending.length}`);

// Sign a specific request
await client.signRequest(requestId);
```

**Via CLI:**
```bash
# View pending requests
dfx canister call chainguard get_pending_requests --network ic

# Sign request as authorized principal
dfx canister call chainguard sign_request '(1)' --network ic

# Check request status
dfx canister call chainguard get_pending_request '(1)' --network ic
```

### Integration with Frontend

The Next.js dashboard provides a visual interface for threshold signature management:
- View all pending requests with progress bars
- See required vs current signatures
- Approve requests with one click
- Real-time status updates

Visit `http://localhost:3000/signatures` when running the frontend.

## Development

### Project Structure

```
ai-agent/
├── src/
│   ├── index.ts                    # Main entry point
│   ├── types/
│   │   ├── chainguard.ts           # Type definitions
│   │   └── idl.ts                  # Candid IDL factory
│   ├── utils/
│   │   ├── chainguard-client.ts    # Canister client wrapper
│   │   └── config.ts               # Configuration manager
│   ├── strategies/
│   │   ├── dca.ts                  # DCA strategy
│   │   └── rebalance.ts            # Rebalancing strategy
│   └── examples/
│       └── demo.ts                 # Comprehensive demo
├── config.example.yaml             # Example configuration
├── package.json
└── tsconfig.json
```

### Building and Running

```bash
# Compile TypeScript to JavaScript
npm run build

# Run compiled code
npm start

# Or run directly with ts-node (development)
npm run dev
```

Compiled output is placed in the `dist/` directory.

### Adding Custom Strategies

Create custom automated strategies by following this pattern:

1. **Create strategy file** in `src/strategies/your-strategy.ts`
2. **Implement core logic** with ChainGuardClient integration
3. **Add configuration** to `config.yaml` and `config.example.yaml`
4. **Create npm script** in `package.json` for easy execution
5. **Test thoroughly** on Sepolia testnet first

**Example Custom Strategy:**

```typescript
// src/strategies/yield-farming.ts
import { ChainGuardClient } from '../utils/chainguard-client';
import { ConfigManager } from '../utils/config';

export class YieldFarmingStrategy {
  constructor(
    private client: ChainGuardClient,
    private config: ConfigManager
  ) {}

  async execute(): Promise<void> {
    console.log('Executing yield farming strategy...');

    // Get strategy parameters from config
    const params = this.config.getStrategy('yieldFarm');

    // Example: Swap USDC for farming token
    const swapResult = await this.client.swap({
      chain: params.chain,
      token_in: params.stableToken,
      token_out: params.farmToken,
      amount_in: params.amount,
      min_amount_out: 1,
    });

    console.log('Swap completed:', swapResult);

    // Additional logic: staking, reward claiming, etc.
  }
}
```

**Add to package.json:**
```json
{
  "scripts": {
    "farm": "ts-node src/strategies/yield-farming.ts"
  }
}
```

## Security Best Practices

**Identity Management**
- Never commit `*.pem` files - Add to `.gitignore`
- Use environment variables for sensitive configuration
- Rotate identities periodically for production agents
- Store production identities in secure key management systems

**Testing & Deployment**
- Always test on Sepolia testnet before mainnet deployment
- Start with small amounts when testing new strategies
- Verify contract addresses and token decimals
- Monitor gas prices to avoid expensive transactions

**Policy Configuration**
- Set conservative amount limits for automated strategies
- Use daily/hourly limits to prevent runaway execution
- Require threshold signatures for amounts above certain thresholds
- Regularly review and audit policy effectiveness

**Monitoring & Alerts**
- Regularly inspect audit logs for unexpected behavior
- Set up alerts for failed transactions
- Monitor canister cycles balance
- Track strategy performance metrics

**Multi-Signature Requirements**
- Use threshold signatures for large transactions (>$1000 equivalent)
- Require multiple roles for critical operations
- Maintain separation of duties between strategy execution and approval

## Troubleshooting

### Identity Issues

**"Failed to load identity"**
- Ensure `identity.pem` exists in the agent directory
- Check file permissions: `chmod 600 identity.pem`
- Agent auto-generates temporary identity if file not found
- For production, export your dfx identity: `dfx identity export default > identity.pem`

### Connection Errors

**"Connection refused" or "Canister not found"**
- Verify canister ID in `config.yaml` matches deployed canister
- Check network setting: `"ic"` for mainnet, `"local"` for local replica
- For local development: ensure `dfx start --background` is running
- Test connection: `dfx canister call chainguard get_config --network ic`

### Policy Rejections

**"Action denied by policy"**
- List current policies: `dfx canister call chainguard list_policies --network ic`
- Verify your identity has required role (Owner, Operator)
- Check action parameters match policy conditions (amount, chain, tokens)
- Review audit logs for detailed rejection reason
- Ensure policies exist - if none match, actions are denied by default

### Canister Errors

**"Insufficient cycles"**
- Canister requires cycles for EVM RPC calls (~10B cycles per call)
- Check cycles balance: `dfx canister status chainguard --network ic`
- Top up canister: `dfx cycles convert --amount 1.0 --network ic && dfx canister deposit-cycles 1000000000000 chainguard --network ic`

**"Transaction failed"**
- Check canister has sufficient ETH/tokens for transaction
- Verify gas estimation didn't fail (high network congestion)
- Review audit logs for detailed error message
- Ensure token addresses are correct for the chain

### Strategy Execution Issues

**DCA not executing**
- Verify source token balance in canister wallet
- Check token approval for swap contract
- Ensure policies allow the swap amount
- Review audit logs for failed attempts

**Rebalancing not triggering**
- Verify deviation exceeds `rebalanceThreshold`
- Check all portfolio tokens have balances
- Ensure policies allow required swap amounts
- Review configuration syntax in `config.yaml`

## Testing

```bash
# Run comprehensive demo (recommended first step)
npm run test

# Test individual strategies
npm run dca       # Dollar Cost Averaging
npm run rebalance # Portfolio Rebalancing

# Development mode with auto-reload
npm run dev
```

## Example Walkthrough

The `src/examples/demo.ts` file provides a comprehensive demonstration of all agent capabilities:

**What the demo covers:**
1. **Client Initialization** - Connecting to ChainGuard canister on IC mainnet
2. **Configuration Inspection** - Reading canister settings and supported chains
3. **Policy-Based Execution** - Submitting transactions and observing policy evaluation
4. **Threshold Signature Workflow** - Creating multi-sig requests and approval process
5. **DCA Strategy** - Automated USDC → ETH purchases
6. **Rebalancing Strategy** - Portfolio maintenance with target allocations
7. **Audit Log Inspection** - Reviewing complete transaction history

**Run the demo:**
```bash
npm run test
```

The demo provides console output showing each step, making it ideal for understanding the complete ChainGuard + AI Agent workflow.

## Production Deployment

**Deployed Canister:**
- Canister ID: `foxtk-ziaaa-aaaai-atthq-cai`
- Network: Internet Computer Mainnet
- Chain-Key ECDSA: `test_key_1`
- Supported Chains: Ethereum, Sepolia

**Verified Transactions:**
- Sepolia Contract: `0xfdd0e2016079951225bd88a41b1b7295aa995cad`
- Example TX: `0xfd8d8b026020e08b06f575702661a76a074c6e34d23f326d84395fec0f9240ad`

## Additional Resources

**ChainGuard SDK**
- [Main Documentation](../../README.md)
- [Canister Source Code](../../src/chainguard/)
- [CLAUDE.md - Project Instructions](../../CLAUDE.md)

**Frontend Dashboard**
- [Frontend Source](../../frontend/)
- Location: `http://localhost:3000` when running locally
- Deployed: TBD

**Internet Computer**
- [ICP JavaScript Agent](https://www.npmjs.com/package/@dfinity/agent)
- [ICP Developer Documentation](https://internetcomputer.org/docs)
- [Chain-Key ECDSA Guide](https://internetcomputer.org/docs/current/developer-docs/integrations/t-ecdsa/)
- [Developer Forum](https://forum.dfinity.org/)

**DeFi Protocols**
- [Uniswap V3 Documentation](https://docs.uniswap.org/protocol/introduction)
- [EIP-1559 Transaction Format](https://eips.ethereum.org/EIPS/eip-1559)
- [Ethereum JSON-RPC Specification](https://ethereum.org/en/developers/docs/apis/json-rpc/)

## License

MIT

## Support

For issues, questions, or contributions:
- **GitHub Issues**: Report bugs or request features
- **ICP Forum**: [Internet Computer Developer Forum](https://forum.dfinity.org/)
- **Documentation**: See [CLAUDE.md](../../CLAUDE.md) for comprehensive project guide
