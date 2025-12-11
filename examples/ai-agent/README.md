# ChainGuard AI Agent

**Autonomous DeFi agent for secure multi-chain transactions on ICP**

This AI agent demonstrates how to build automated trading strategies using the ChainGuard SDK with policy-based access control, threshold signatures, and complete auditability.

## Features

- **🤖 Automated Strategies**
  - Dollar Cost Averaging (DCA)
  - Portfolio Rebalancing
  - Extensible architecture for custom strategies

- **🔐 Security First**
  - Policy-based execution control
  - Threshold signature approval for large transactions
  - Complete audit trail
  - Role-based access control

- **⛓️ Multi-Chain Support**
  - Ethereum mainnet
  - Sepolia testnet
  - Extensible to other EVM chains

- **📊 Monitoring & Analytics**
  - Real-time strategy performance tracking
  - Audit log inspection
  - Pending request monitoring

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

## Usage

### Run Complete Demo

Execute all features in a single comprehensive demo:

```bash
npm run test
```

This runs the demo script (`src/examples/demo.ts`) which showcases:
- Policy-based transaction execution
- Threshold signature workflow
- DCA strategy
- Portfolio rebalancing
- Audit log inspection

### Execute Strategies

```bash
# Run DCA strategy once
npm run dca

# Run rebalancing strategy once
npm run rebalance

# Execute all strategies once
npm start all
```

### Scheduled Execution

Run strategies on a schedule (configured in config.yaml):

```bash
npm start schedule
```

The agent will run continuously and execute strategies at scheduled intervals.

### Monitoring

```bash
# View agent status
npm start status

# Monitor pending threshold signature requests
npm start monitor
```

## Strategies

### Dollar Cost Averaging (DCA)

Automatically purchases a fixed amount of a target token at regular intervals, regardless of price. This reduces the impact of volatility.

**Configuration:**
```yaml
dca:
  enabled: true
  interval: "0 0 * * *"  # Cron format
  params:
    sourceToken: "USDC"
    targetToken: "ETH"
    amountPerPurchase: "1000000"  # In smallest unit
    chain: "Sepolia"
```

**Benefits:**
- Reduces timing risk
- Automates accumulation
- Emotion-free investing

### Portfolio Rebalancing

Maintains target asset allocations by automatically swapping when deviations exceed a threshold.

**Configuration:**
```yaml
rebalance:
  enabled: true
  interval: "0 0 * * 0"  # Weekly
  params:
    chain: "Sepolia"
    rebalanceThreshold: 5  # 5% deviation triggers rebalance
    portfolio:
      - token: "ETH"
        targetPercentage: 50
      - token: "USDC"
        targetPercentage: 30
      - token: "WETH"
        targetPercentage: 20
```

**Benefits:**
- Maintains risk profile
- Automatic profit-taking
- Disciplined execution

## Policy Integration

The agent respects ChainGuard policies for all transactions:

### Example Policies

**Auto-approve small transactions:**
```yaml
name: "Allow Small Transfers"
conditions:
  - MaxAmount: 1000000000000000000  # 1 ETH
  - AllowedChains: ["Sepolia"]
action: Allow
priority: 1
```

**Require threshold for large transactions:**
```yaml
name: "Threshold for Large Transfers"
conditions:
  - MinAmount: 1000000000000000000  # > 1 ETH
action:
  RequireThreshold:
    required: 2
    from_roles: [Operator, Owner]
priority: 2
```

## Threshold Signature Workflow

When a transaction requires threshold signatures:

1. **Agent requests action** → ChainGuard evaluates policies
2. **Policy requires threshold** → Request enters pending state
3. **Multiple signers approve** → Each calls `sign_request(id)`
4. **Threshold met** → Transaction executes automatically
5. **Result logged** → Audit trail updated

### Signing Pending Requests

From the agent:
```typescript
// Get pending requests
const pending = await client.getPendingRequests();

// Sign a request
await client.signRequest(requestId);
```

From dfx CLI:
```bash
dfx canister call chainguard sign_request '(1)' --network ic
```

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

### Building

```bash
# Compile TypeScript
npm run build

# Output will be in dist/
```

### Adding Custom Strategies

1. Create a new file in `src/strategies/`
2. Implement the strategy class with `execute()` method
3. Use `ChainGuardClient` to interact with the canister
4. Add configuration to `config.yaml`
5. Register in `src/index.ts`

Example:

```typescript
export class CustomStrategy {
  constructor(
    private client: ChainGuardClient,
    private config: ConfigManager
  ) {}

  async execute(): Promise<ActionResult> {
    // Your strategy logic
    return await this.client.swap(...);
  }
}
```

## Security Best Practices

1. **Never commit identity files** - Add `*.pem` to `.gitignore`
2. **Use environment variables** - Store sensitive config in `.env`
3. **Start with testnet** - Test on Sepolia before mainnet
4. **Set conservative limits** - Configure `maxTransactionSize` and `dailyLimit`
5. **Monitor audit logs** - Regularly review transaction history
6. **Use threshold signatures** - Require multi-sig for large amounts

## Troubleshooting

### "Failed to load identity"
- Ensure `identity.pem` exists and has correct permissions
- Agent will auto-generate temporary identity if not found

### "Connection refused"
- Check canister ID is correct
- Verify network setting (ic vs local)
- For local: ensure `dfx start` is running

### "Action denied by policy"
- Review policies with `dfx canister call chainguard list_policies`
- Check if your identity has required roles
- Verify action parameters match policy conditions

### "Insufficient cycles"
- Canister needs cycles for EVM RPC calls
- Top up with: `dfx cycles convert --amount 1.0`

## Testing

```bash
# Run the comprehensive demo
npm run test

# Execute specific strategy
npm run dca
npm run rebalance

# Check agent status
npm start status
```

## Examples

See `src/examples/demo.ts` for a complete walkthrough of:
- Client initialization
- Policy-based execution
- Threshold signature workflow
- Strategy execution
- Audit log inspection

## License

MIT

## Support

For issues or questions:
- GitHub Issues: [ChainGuard SDK Issues](https://github.com/carlos-israelj/ChainGuard-SDK/issues)
- ICP Forum: [Internet Computer Developer Forum](https://forum.dfinity.org/)

## Resources

- [ChainGuard SDK Documentation](../../README.md)
- [ICP JavaScript Agent](https://www.npmjs.com/package/@dfinity/agent)
- [ICP Developer Docs](https://internetcomputer.org/docs)
- [Uniswap V3 Documentation](https://docs.uniswap.org/protocol/introduction)
