#!/bin/bash
# Bitcoin Integration Testing Script
# This script tests the complete Bitcoin integration on IC testnet

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
export PATH="$HOME/.local/share/dfx/bin:$PATH"
CANISTER_ID="foxtk-ziaaa-aaaai-atthq-cai"
NETWORK="ic"
BTC_NETWORK="BitcoinTestnet"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  ChainGuard Bitcoin Integration Test Suite${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Step 1: Check canister deployment
echo -e "${YELLOW}[1/7]${NC} Checking canister deployment..."
if dfx canister status chainguard --network $NETWORK &>/dev/null; then
    echo -e "${GREEN}✓${NC} Canister is deployed: $CANISTER_ID"
else
    echo -e "${RED}✗${NC} Canister not deployed. Please deploy first:"
    echo "      dfx deploy chainguard --network ic"
    exit 1
fi
echo ""

# Step 2: Check if canister is running
echo -e "${YELLOW}[2/7]${NC} Checking canister status..."
STATUS=$(dfx canister status chainguard --network $NETWORK 2>&1)
if echo "$STATUS" | grep -q "Running"; then
    echo -e "${GREEN}✓${NC} Canister is running"
else
    echo -e "${RED}✗${NC} Canister is not running"
    echo "$STATUS"
    exit 1
fi
echo ""

# Step 3: Get Bitcoin address
echo -e "${YELLOW}[3/7]${NC} Getting Bitcoin testnet address..."
BTC_ADDRESS=$(dfx canister call chainguard get_bitcoin_address "(\"$BTC_NETWORK\")" --network $NETWORK 2>&1)

if echo "$BTC_ADDRESS" | grep -q "Ok"; then
    # Extract address from response
    ADDRESS=$(echo "$BTC_ADDRESS" | grep -oP '(?<=")tb1[^"]+' || echo "$BTC_ADDRESS" | grep -oP '(?<=")bc1[^"]+')
    echo -e "${GREEN}✓${NC} Bitcoin Address: ${BLUE}$ADDRESS${NC}"
    echo ""
    echo -e "${YELLOW}📝 IMPORTANT:${NC} Fund this address with testnet BTC"
    echo "   Faucets:"
    echo "   • https://coinfaucet.eu/en/btc-testnet/"
    echo "   • https://testnet-faucet.mempool.co/"
    echo "   • https://bitcoinfaucet.uo1.net/"
    echo ""
    echo -e "   Minimum: ${BLUE}0.001 BTC (100,000 satoshis)${NC}"
    echo -e "   Block explorer: https://blockstream.info/testnet/address/$ADDRESS"
else
    echo -e "${RED}✗${NC} Failed to get Bitcoin address"
    echo "$BTC_ADDRESS"
    exit 1
fi
echo ""

# Step 4: Wait for user to fund address
echo -e "${YELLOW}[4/7]${NC} Waiting for funding..."
read -p "$(echo -e ${YELLOW}Press Enter after funding the address...${NC})"

# Step 5: Check configuration
echo -e "${YELLOW}[5/7]${NC} Checking canister configuration..."
CONFIG=$(dfx canister call chainguard get_config --network $NETWORK 2>&1)
if echo "$CONFIG" | grep -q "opt"; then
    echo -e "${GREEN}✓${NC} Configuration exists"
else
    echo -e "${YELLOW}⚠${NC}  No configuration found. Initializing..."

    # Initialize with basic config
    dfx canister call chainguard initialize '(
      record {
        name = "ChainGuard Bitcoin Test";
        default_threshold = record { required = 1; total = 1; };
        supported_chains = vec { "Sepolia"; "Bitcoin"; "BitcoinTestnet" };
        policies = vec {
          record {
            name = "Allow Bitcoin Transfers";
            conditions = vec {
              variant { MaxAmount = 1000000 };
              variant { AllowedChains = vec { "BitcoinTestnet"; "Bitcoin" } };
            };
            action = variant { Allow };
            priority = 1;
          }
        };
      }
    )' --network $NETWORK

    echo -e "${GREEN}✓${NC} Configuration initialized"
fi
echo ""

# Step 6: Execute test transfer
echo -e "${YELLOW}[6/7]${NC} Executing test Bitcoin transfer..."
echo -e "   Amount: ${BLUE}10,000 satoshis (0.0001 BTC)${NC}"

# Use a well-known testnet address for testing
TEST_RECIPIENT="tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx"
echo -e "   Recipient: ${BLUE}$TEST_RECIPIENT${NC}"
echo ""

RESULT=$(dfx canister call chainguard request_action "(
  variant {
    BitcoinTransfer = record {
      to = \"$TEST_RECIPIENT\";
      amount = 10000;
      network = \"$BTC_NETWORK\"
    }
  }
)" --network $NETWORK 2>&1)

echo "$RESULT"
echo ""

# Parse result
if echo "$RESULT" | grep -q "Executed"; then
    echo -e "${GREEN}✓${NC} Transaction executed successfully!"

    # Try to extract TX hash
    if echo "$RESULT" | grep -q "tx_hash"; then
        TX_HASH=$(echo "$RESULT" | grep -oP '(?<=tx_hash = opt ")[^"]+' || echo "N/A")
        if [ "$TX_HASH" != "N/A" ]; then
            echo -e "   TX Hash: ${BLUE}$TX_HASH${NC}"
            echo -e "   Explorer: https://blockstream.info/testnet/tx/$TX_HASH"
        fi
    fi

elif echo "$RESULT" | grep -q "PendingSignatures"; then
    echo -e "${YELLOW}⏳${NC} Transaction requires threshold signatures"
    REQUEST_ID=$(echo "$RESULT" | grep -oP '(?<=id = )[0-9]+' || echo "unknown")
    echo -e "   Request ID: $REQUEST_ID"

elif echo "$RESULT" | grep -q "Denied"; then
    echo -e "${RED}✗${NC} Transaction denied by policy"
    REASON=$(echo "$RESULT" | grep -oP '(?<=reason = ")[^"]+' || echo "unknown")
    echo -e "   Reason: $REASON"

else
    echo -e "${RED}✗${NC} Unknown result or error"
fi
echo ""

# Step 7: Check audit logs
echo -e "${YELLOW}[7/7]${NC} Checking audit logs..."
AUDIT=$(dfx canister call chainguard get_audit_logs '(null, null)' --network $NETWORK 2>&1)

# Count entries
ENTRY_COUNT=$(echo "$AUDIT" | grep -c "id = " || echo "0")
echo -e "${GREEN}✓${NC} Found $ENTRY_COUNT audit entries"

if [ "$ENTRY_COUNT" -gt 0 ]; then
    echo ""
    echo -e "${BLUE}Latest audit entries:${NC}"
    echo "$AUDIT" | grep -A 5 "action_type = " | head -20
fi

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✓${NC} Bitcoin integration test completed!"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Summary
echo -e "${YELLOW}Summary:${NC}"
echo "  • Canister: $CANISTER_ID"
echo "  • Bitcoin Address: $ADDRESS"
echo "  • Test Recipient: $TEST_RECIPIENT"
echo "  • Network: $BTC_NETWORK"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "  1. Check transaction on block explorer"
echo "  2. Wait ~10-60 minutes for confirmation"
echo "  3. Verify balance changes"
echo "  4. Review audit logs for complete history"
echo ""
