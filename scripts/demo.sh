#!/bin/bash

# ChainGuard SDK - Demo Script

set -e

export PATH="$HOME/.local/share/dfx/bin:$PATH"

echo "🎬 ChainGuard SDK Demo"
echo "====================="

# Get current principal
PRINCIPAL=$(dfx identity get-principal)
echo "Current principal: $PRINCIPAL"

echo ""
echo "📋 Step 1: Check configuration"
dfx canister call chainguard get_config

echo ""
echo "📋 Step 2: Request a small transfer (should be allowed)"
dfx canister call chainguard request_action '(
  variant {
    Transfer = record {
      chain = "ethereum";
      token = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
      to = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0";
      amount = 500000000
    }
  }
)'

echo ""
echo "📋 Step 3: Request a large transfer (should require threshold)"
dfx canister call chainguard request_action '(
  variant {
    Transfer = record {
      chain = "ethereum";
      token = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
      to = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0";
      amount = 5000000000
    }
  }
)'

echo ""
echo "📋 Step 4: Get pending requests"
dfx canister call chainguard get_pending_requests

echo ""
echo "📋 Step 5: View audit logs"
dfx canister call chainguard get_audit_logs '(null, null)'

echo ""
echo "✅ Demo complete!"
echo ""
echo "Key features demonstrated:"
echo "- Policy evaluation (allowed vs. threshold required)"
echo "- Audit trail logging"
echo "- Threshold signature workflow"
