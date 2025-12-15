# ChainGuard Frontend Dashboard

> Next.js 16.0.8 dashboard for ChainGuard - Security middleware for AI agents on Internet Computer Protocol

[![Next.js](https://img.shields.io/badge/Next.js-16.0.8-black?logo=next.js)](https://nextjs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5-blue?logo=typescript)](https://www.typescriptlang.org/)
[![TailwindCSS](https://img.shields.io/badge/TailwindCSS-4-38bdf8?logo=tailwindcss)](https://tailwindcss.com/)

## Overview

The ChainGuard Frontend is a professional web interface for managing multi-chain AI agent transactions with role-based access control, threshold signatures, and complete auditability. Built with Next.js 16.0.8 App Router and the ChainGuard TypeScript SDK.

## Recent Updates

**December 15, 2024 - SDK v0.1.1 Integration**
- ✅ Migrated to `@chainguarsdk/sdk` v0.1.1
- ✅ Removed duplicate type definitions (now use SDK types)
- ✅ Updated `useChainGuard` hook to use SDK client
- ✅ Compatible with canister stable memory implementation
- ✅ Downgraded @dfinity/* packages to v2.1.1 for compatibility
- ✅ Build successful with zero errors

## Features

### 📊 Dashboard Pages

1. **Home Dashboard** (`/`)
   - System overview and statistics
   - Quick actions and Getting Started guide
   - System configuration display
   - Key features showcase

2. **Strategies** (`/strategies`)
   - Configure DCA (Dollar Cost Averaging)
   - Portfolio Rebalancing setup
   - Execution history and monitoring
   - Real-time strategy status

3. **Threshold Signatures** (`/signatures`)
   - View pending signature requests
   - Approve/reject multi-sig transactions
   - Signature status tracking
   - Real-time updates

4. **Audit Logs** (`/audit`)
   - Complete transaction history
   - Filter by date range
   - Export to CSV
   - Policy evaluation details

### 🎨 Design System

- **ICP Color Palette**: Purple (#3B00B9), Pink (#ED1E79), Teal (#18C39F)
- **Clean UI**: Border-based card design, no heavy shadows
- **Hero Sections**: Large typography with gradient accents
- **Responsive**: Mobile-first design with Tailwind CSS
- **Professional**: Documentation-style layout

## Quick Start

### Prerequisites

- Node.js 18+
- npm or yarn
- ChainGuard canister deployed on IC mainnet

### Installation

```bash
# Navigate to frontend directory
cd frontend

# Install dependencies
npm install

# Run development server
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) to view the dashboard.

### Build for Production

```bash
# Create optimized production build
npm run build

# Start production server
npm start
```

### Linting

```bash
npm run lint
```

## Architecture

### Technology Stack

```
Frontend Stack:
├── Next.js 16.0.8 (App Router)
├── React 19.2.1
├── TypeScript 5
├── TailwindCSS 4
├── @chainguarsdk/sdk 0.1.1
├── @dfinity/agent 2.1.1
├── lucide-react (icons)
└── recharts (charts)
```

### Project Structure

```
frontend/
├── app/                      # Next.js App Router pages
│   ├── page.tsx             # Dashboard home
│   ├── strategies/          # Strategy configuration
│   ├── signatures/          # Threshold approvals
│   └── audit/               # Audit log viewer
├── components/              # React components
│   └── NavigationLayout.tsx
├── lib/
│   └── hooks/
│       └── useChainGuard.ts # ChainGuard SDK integration
├── public/                  # Static assets
└── package.json
```

### useChainGuard Hook

The `useChainGuard` hook provides a React-friendly interface to the ChainGuard SDK:

```typescript
import { useChainGuard } from '@/lib/hooks/useChainGuard';

function MyComponent() {
  const {
    client,          // ChainGuardClient instance
    principal,       // Current user principal
    loading,         // Connection state
    error,           // Error messages
    // Methods
    transfer,
    swap,
    approveToken,
    getPendingRequests,
    signRequest,
    getAuditLogs,
    listPolicies,
    getConfig,
    isPaused,
  } = useChainGuard();

  // Use the hook methods
  const handleTransfer = async () => {
    const result = await transfer(
      'Sepolia',
      'ETH',
      '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0',
      BigInt(1000000000000000)
    );
    console.log(result);
  };
}
```

## Configuration

### Canister Connection

Edit `/lib/hooks/useChainGuard.ts` to configure:

```typescript
const CANISTER_ID = 'foxtk-ziaaa-aaaai-atthq-cai';  // ChainGuard canister
const HOST = 'https://icp-api.io';                   // IC mainnet
```

### Identity Management

**Current Setup (Development):**
- Auto-generated Ed25519 identity per session
- Identity regenerates on page refresh

**Production Recommendations:**
- Integrate with [Internet Identity](https://identity.ic0.app/)
- Use [@dfinity/auth-client](https://www.npmjs.com/package/@dfinity/auth-client)
- Store identity in browser localStorage
- Implement proper session management

Example Internet Identity integration:

```typescript
import { AuthClient } from '@dfinity/auth-client';

const authClient = await AuthClient.create();
await authClient.login({
  identityProvider: 'https://identity.ic0.app',
  onSuccess: () => {
    const identity = authClient.getIdentity();
    // Use this identity with ChainGuardClient
  },
});
```

## Development

### Environment Variables

Create `.env.local` for local development:

```bash
NEXT_PUBLIC_CANISTER_ID=foxtk-ziaaa-aaaai-atthq-cai
NEXT_PUBLIC_IC_HOST=https://icp-api.io
```

### Available Scripts

| Command | Description |
|---------|-------------|
| `npm run dev` | Start development server (port 3000) |
| `npm run build` | Create production build |
| `npm start` | Start production server |
| `npm run lint` | Run ESLint |

### Adding New Pages

1. Create page in `app/` directory
2. Use `useChainGuard` hook for canister interaction
3. Follow ICP design system guidelines
4. Add navigation link in `NavigationLayout.tsx`

## Deployment

### Vercel (Recommended)

1. Push code to GitHub
2. Import project to [Vercel](https://vercel.com)
3. Configure environment variables
4. Deploy

### Self-Hosted

```bash
npm run build
npm start
```

Server runs on port 3000 by default.

## API Integration

The frontend uses the `@chainguarsdk/sdk` package for all canister interactions:

```typescript
// All methods available via useChainGuard hook
const { transfer, swap, getPendingRequests, getAuditLogs } = useChainGuard();

// Execute transfer
await transfer('Sepolia', 'ETH', recipientAddress, amount);

// Execute swap
await swap('Sepolia', tokenIn, tokenOut, amountIn, minAmountOut);

// Get pending requests
const pending = await getPendingRequests();

// Get audit logs
const logs = await getAuditLogs();
```

## Verified Transactions

Test the frontend with verified Sepolia transactions:
- **ETH Transfer**: [0xfd8d8b...240ad](https://sepolia.etherscan.io/tx/0xfd8d8b026020e08b06f575702661a76a074c6e34d23f326d84395fec0f9240ad)
- **ETH→USDC Swap**: [0x9c30a3...f4db9](https://sepolia.etherscan.io/tx/0x9c30a38f4e0f58bc1dd29c34c5e3f7c31d8dc3f7bab8d31dc0e3ec5eae0f4db9)
- **USDC→ETH Swap**: [0xbfbdab...8652a](https://sepolia.etherscan.io/tx/0xbfbdab70dd24fcb72c70b60f94096c67ca5cf949e3e99d201ba088377ed8652a)

## Troubleshooting

### Build Errors

**Issue**: TypeScript errors during build
**Solution**: Ensure all @dfinity packages are v2.1.1 and @chainguarsdk/sdk is v0.1.1

**Issue**: Module not found errors
**Solution**: Run `npm install` to ensure all dependencies are installed

### Runtime Errors

**Issue**: "Failed to initialize ChainGuard client"
**Solution**: Check canister ID and network configuration

**Issue**: Identity errors
**Solution**: Clear browser cache and regenerate identity

## Resources

**Project Documentation**
- [Main README](../README.md)
- [CLAUDE.md - Development Guide](../CLAUDE.md)
- [AI Agent Examples](../examples/ai-agent/)

**ChainGuard SDK**
- [SDK Package](../packages/sdk/)
- [npm Package](https://www.npmjs.com/package/@chainguarsdk/sdk)

**Next.js**
- [Next.js Documentation](https://nextjs.org/docs)
- [Next.js GitHub](https://github.com/vercel/next.js)

**Internet Computer**
- [ICP Developer Docs](https://internetcomputer.org/docs)
- [dfinity/agent Documentation](https://agent-js.icp.xyz/)

## License

MIT

## Support

For issues or questions:
- **GitHub Issues**: [ChainGuard SDK Issues](https://github.com/carlos-israelj/ChainGuard-SDK/issues)
- **ICP Forum**: [Developer Forum](https://forum.dfinity.org/)
- **Documentation**: See [CLAUDE.md](../CLAUDE.md)
