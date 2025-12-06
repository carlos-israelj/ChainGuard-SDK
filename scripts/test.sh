#!/bin/bash

# ChainGuard SDK - Testing Script

set -e

export PATH="$HOME/.local/share/dfx/bin:$PATH"

echo "🧪 ChainGuard SDK Tests"
echo "======================="

# Run Rust unit tests
echo ""
echo "Running Rust unit tests..."
cargo test

# Build to check compilation
echo ""
echo "Checking compilation..."
cargo build --target wasm32-unknown-unknown --release

# If dfx is running, do integration tests
if dfx ping > /dev/null 2>&1; then
    echo ""
    echo "Running integration tests..."

    # Check if canister is deployed
    if dfx canister id chainguard > /dev/null 2>&1; then
        echo "Testing get_config..."
        dfx canister call chainguard get_config

        echo ""
        echo "Testing is_paused..."
        dfx canister call chainguard is_paused

        echo ""
        echo "Testing list_policies..."
        dfx canister call chainguard list_policies

        echo ""
        echo "✅ Integration tests passed!"
    else
        echo "⚠️  Canister not deployed. Skipping integration tests."
    fi
else
    echo "⚠️  Local replica not running. Skipping integration tests."
fi

echo ""
echo "✅ All tests completed!"
