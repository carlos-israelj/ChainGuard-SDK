# @chainguarsdk/sdk

> TypeScript SDK for ChainGuard - Security middleware for AI agents on Internet Computer Protocol

[![npm version](https://img.shields.io/npm/v/@chainguarsdk/sdk.svg?style=flat-square)](https://www.npmjs.com/package/@chainguarsdk/sdk)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](../../LICENSE)

## Installation

```bash
npm install @chainguarsdk/sdk
```

## Quick Start

```typescript
import { ChainGuardClient } from '@chainguarsdk/sdk';

// Initialize client
const client = new ChainGuardClient({
  canisterId: 'foxtk-ziaaa-aaaai-atthq-cai',
  host: 'https://icp-api.io', // Optional, defaults to IC mainnet
});

// Execute a transfer
const result = await client.transfer(
  'Sepolia',           // chain
  'ETH',               // token
  '0x742d35Cc...',     // recipient address
  BigInt(1000000)      // amount in wei
);

console.log(result); // ActionResult (Executed | PendingSignatures | Denied)
```

## Features

- **Simple API** - Clean, intuitive methods for all ChainGuard operations
- **Type Safety** - Full TypeScript support with complete type definitions
- **Flexible Authentication** - Bring your own identity or use auto-generated ones
- **Helper Methods** - Convenient wrappers for common actions (transfer, swap, approve)
- **Complete Coverage** - Access to all canister functionality

## API Reference

### Initialization

```typescript
import { ChainGuardClient } from '@chainguarsdk/sdk';
import { Ed25519KeyIdentity } from '@dfinity/identity';

// Option 1: Auto-generated identity (for testing)
const client = new ChainGuardClient({
  canisterId: 'your-canister-id',
});

// Option 2: Provide your own identity
const identity = Ed25519KeyIdentity.fromSecretKey(yourSecretKey);
const client = new ChainGuardClient({
  canisterId: 'your-canister-id',
  identity,
});

// Option 3: Custom host (for local development)
const client = new ChainGuardClient({
  canisterId: 'your-canister-id',
  host: 'http://localhost:4943',
});
```

### Actions

#### Transfer Tokens

```typescript
const result = await client.transfer(
  'Sepolia',              // chain
  'ETH',                  // token
  '0x742d35Cc6634C0...',  // to address
  BigInt(1000000000000000)  // 0.001 ETH in wei
);

// Check result
if ('Executed' in result) {
  console.log('Transaction hash:', result.Executed.tx_hash[0]);
} else if ('PendingSignatures' in result) {
  console.log('Awaiting signatures:', result.PendingSignatures.id);
} else if ('Denied' in result) {
  console.log('Denied:', result.Denied.reason);
}
```

#### Swap Tokens

```typescript
const result = await client.swap(
  'Sepolia',                      // chain
  '0x1c7D4B196Cb0C7B01d743...',  // token in (USDC)
  'ETH',                          // token out
  BigInt(1000000),                // amount in (1 USDC with 6 decimals)
  BigInt(1),                      // min amount out
  3000                            // fee tier (0.3%) - optional
);
```

#### Approve Token Spending

```typescript
const result = await client.approveToken(
  'Sepolia',
  '0x1c7D4B196Cb0C7B01d743...',  // token address
  '0x68b3465833fb72A70...',      // spender (Uniswap router)
  BigInt(1000000)                 // amount to approve
);
```

### Threshold Signatures

#### Get Pending Requests

```typescript
const pendingRequests = await client.getPendingRequests();

for (const request of pendingRequests) {
  console.log(`Request ${request.id}:`);
  console.log(`  Required signatures: ${request.required_signatures}`);
  console.log(`  Collected: ${request.collected_signatures.length}`);
  console.log(`  Status:`, request.status);
}
```

#### Sign a Request

```typescript
const result = await client.signRequest(BigInt(1)); // request ID

if ('Ok' in result) {
  console.log('Signature added successfully');
  const updatedRequest = result.Ok;
  console.log(`Signatures: ${updatedRequest.collected_signatures.length}/${updatedRequest.required_signatures}`);
} else {
  console.error('Error:', result.Err);
}
```

#### Reject a Request

```typescript
const result = await client.rejectRequest(
  BigInt(1),                    // request ID
  'Invalid recipient address'  // reason
);
```

### Policies

#### List Policies

```typescript
const policies = await client.listPolicies();

for (const policy of policies) {
  console.log(`Policy: ${policy.name}`);
  console.log(`  Priority: ${policy.priority}`);
  console.log(`  Conditions:`, policy.conditions);
  console.log(`  Action:`, policy.action);
}
```

#### Add a Policy

```typescript
const result = await client.addPolicy({
  name: 'Allow Small Swaps',
  conditions: [
    { MaxAmount: BigInt(10000000) },  // Max 10 USDC
    { AllowedChains: ['Sepolia'] },
  ],
  action: { Allow: null },
  priority: 1,
});

if ('Ok' in result) {
  console.log('Policy added with ID:', result.Ok);
}
```

### Audit Logs

#### Get Audit Logs

```typescript
// Get all audit logs
const logs = await client.getAuditLogs();

// Get logs in time range (timestamps in nanoseconds)
const start = BigInt(Date.now() * 1000000);
const end = BigInt((Date.now() + 86400000) * 1000000); // +24h
const recentLogs = await client.getAuditLogs(start, end);

for (const log of logs) {
  console.log(`[${new Date(Number(log.timestamp) / 1000000)}]`);
  console.log(`  Action: ${log.action_type}`);
  console.log(`  Requester: ${log.requester.toText()}`);
  console.log(`  Decision:`, log.policy_result.decision);

  if (log.execution_result.length > 0) {
    const exec = log.execution_result[0];
    console.log(`  Success: ${exec.success}`);
    if (exec.tx_hash.length > 0) {
      console.log(`  TX Hash: ${exec.tx_hash[0]}`);
    }
  }
}
```

### Role Management

```typescript
import { Principal } from '@dfinity/principal';

// Assign a role
const operatorPrincipal = Principal.fromText('xxxxx-xxxxx-xxx...');
await client.assignRole(operatorPrincipal, { Operator: null });

// Get roles for a principal
const roles = await client.getRoles(operatorPrincipal);
console.log('Roles:', roles);

// Revoke a role
await client.revokeRole(operatorPrincipal, { Operator: null });
```

### Emergency Controls

```typescript
// Pause the system (Owner only)
await client.pause();

// Check if paused
const isPaused = await client.isPaused();
console.log('System paused:', isPaused);

// Resume operations (Owner only)
await client.resume();
```

### Configuration

```typescript
// Get current configuration
const config = await client.getConfig();

if (config) {
  console.log('Canister name:', config.name);
  console.log('Supported chains:', config.supported_chains);
  console.log('Default threshold:', config.default_threshold);
  console.log('Policies:', config.policies.length);
}
```

## Usage Examples

### AI Agent Integration

```typescript
import { ChainGuardClient } from '@chainguarsdk/sdk';

class TradingBot {
  constructor(private guard: ChainGuardClient) {}

  async executeDCA(tokenIn: string, tokenOut: string, amount: bigint) {
    console.log(`Executing DCA: ${amount} ${tokenIn} → ${tokenOut}`);

    const result = await this.guard.swap(
      'Sepolia',
      tokenIn,
      tokenOut,
      amount,
      BigInt(1),  // Accept any amount (for demo)
      3000        // 0.3% fee tier
    );

    if ('Executed' in result && result.Executed.success) {
      console.log('✓ DCA executed:', result.Executed.tx_hash[0]);
      return true;
    } else if ('PendingSignatures' in result) {
      console.log('⏳ Awaiting threshold signatures');
      return false;
    } else if ('Denied' in result) {
      console.error('✗ Denied:', result.Denied.reason);
      return false;
    }
  }
}

// Usage
const guard = new ChainGuardClient({ canisterId: 'your-id' });
const bot = new TradingBot(guard);

await bot.executeDCA(
  '0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238',  // USDC
  'ETH',
  BigInt(1000000)  // 1 USDC
);
```

### Multi-Sig Treasury

```typescript
import { ChainGuardClient } from '@chainguarsdk/sdk';
import { Ed25519KeyIdentity } from '@dfinity/identity';

// Signer 1
const signer1Identity = Ed25519KeyIdentity.generate();
const signer1 = new ChainGuardClient({
  canisterId: 'your-id',
  identity: signer1Identity,
});

// Signer 2
const signer2Identity = Ed25519KeyIdentity.generate();
const signer2 = new ChainGuardClient({
  canisterId: 'your-id',
  identity: signer2Identity,
});

// Request large transfer (will require threshold)
const result = await signer1.transfer(
  'Sepolia',
  'ETH',
  '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0',
  BigInt(10000000000000000000)  // 10 ETH - requires multi-sig
);

if ('PendingSignatures' in result) {
  const requestId = result.PendingSignatures.id;
  console.log('Request created:', requestId);

  // Signer 2 approves
  await signer2.signRequest(requestId);
  console.log('Request signed - will auto-execute when threshold met');
}
```

### React Hook Example

```typescript
import { useState, useEffect } from 'react';
import { ChainGuardClient } from '@chainguarsdk/sdk';

export function useChainGuard(canisterId: string) {
  const [client, setClient] = useState<ChainGuardClient | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const guard = new ChainGuardClient({ canisterId });
    setClient(guard);
    setLoading(false);
  }, [canisterId]);

  return { client, loading };
}

// In your component
function MyComponent() {
  const { client, loading } = useChainGuard('foxtk-ziaaa-aaaai-atthq-cai');

  const handleTransfer = async () => {
    if (!client) return;

    const result = await client.transfer(
      'Sepolia',
      'ETH',
      '0x...',
      BigInt(1000000)
    );

    // Handle result...
  };

  if (loading) return <div>Loading...</div>;

  return <button onClick={handleTransfer}>Send</button>;
}
```

## Type Definitions

The SDK exports all ChainGuard types for TypeScript support:

```typescript
import type {
  Action,
  ActionResult,
  ExecutionResult,
  PendingRequest,
  Policy,
  PolicyAction,
  Condition,
  AuditEntry,
  Role,
  ChainGuardConfig,
} from '@chainguard/sdk';
```

## Error Handling

```typescript
try {
  const result = await client.transfer('Sepolia', 'ETH', '0x...', BigInt(1000));

  if ('Denied' in result) {
    // Handle policy denial
    console.error('Transaction denied:', result.Denied.reason);
  } else if ('PendingSignatures' in result) {
    // Handle threshold requirement
    console.log('Awaiting signatures:', result.PendingSignatures.id);
  } else if ('Executed' in result) {
    // Handle successful execution
    if (result.Executed.success) {
      console.log('Success!', result.Executed.tx_hash[0]);
    } else {
      console.error('Execution failed:', result.Executed.error[0]);
    }
  }
} catch (error) {
  console.error('Network or canister error:', error);
}
```

## Resources

- [ChainGuard Documentation](../../README.md)
- [AI Agent Example](../../examples/ai-agent/)
- [Frontend Dashboard](../../frontend/)
- [ICP Developer Docs](https://internetcomputer.org/docs)

## License

MIT

## Support

- **GitHub Issues**: [Report bugs](https://github.com/carlos-israelj/ChainGuard-SDK/issues)
- **Documentation**: See main [README](../../README.md)
