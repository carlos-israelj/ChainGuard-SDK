#!/bin/bash

# ChainGuard SDK - Deployment Script

set -e

echo "🚀 ChainGuard SDK Deployment"
echo "=============================="

# Add dfx to PATH
export PATH="$HOME/.local/share/dfx/bin:$PATH"

# Check if dfx is running
if ! dfx ping > /dev/null 2>&1; then
    echo "Starting local replica..."
    dfx start --background --clean
    sleep 5
fi

# Build the canister
echo "Building canister..."
cargo build --target wasm32-unknown-unknown --release

# Deploy
echo "Deploying canister..."
dfx deploy chainguard

# Get canister ID
CANISTER_ID=$(dfx canister id chainguard)
echo ""
echo "✅ Deployment successful!"
echo "Canister ID: $CANISTER_ID"
echo ""
echo "Next steps:"
echo "1. Initialize the canister: ./scripts/init_example.sh"
echo "2. Assign roles to principals"
echo "3. Start using ChainGuard!"
