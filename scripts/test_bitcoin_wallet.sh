#!/bin/bash
# Bitcoin Integration Testing Script (with Wallet)
# This script tests the complete Bitcoin integration on IC mainnet using wallet

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
export PATH="$HOME/.local/share/dfx/bin:$PATH"
export DFX_WARNING=-mainnet_plaintext_identity
CANISTER_ID="foxtk-ziaaa-aaaai-atthq-cai"
WALLET_ID="nxwtw-5rpig-7wyhb-tbgzo-5uc7h-qmedn-7mbfu-mn6nr-4uooq-bmkp4-vae"
NETWORK="ic"
BTC_NETWORK="BitcoinTestnet"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  ChainGuard Bitcoin Integration Test Suite${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "Canister ID: ${BLUE}$CANISTER_ID${NC}"
echo -e "Wallet ID:   ${BLUE}$WALLET_ID${NC}"
echo ""

# Step 1: Get Bitcoin address
echo -e "${YELLOW}[1/5]${NC} Getting Bitcoin testnet address..."
BTC_ADDRESS=$(dfx canister call $CANISTER_ID get_bitcoin_address "(\"$BTC_NETWORK\")" --wallet $WALLET_ID --network $NETWORK 2>&1)

if echo "$BTC_ADDRESS" | grep -q "Ok"; then
    # Extract address from response (handle different formats)
    ADDRESS=$(echo "$BTC_ADDRESS" | grep -oP '(?<=")tb1[a-z0-9]+' | head -1)
    if [ -z "$ADDRESS" ]; then
        # Try alternative extraction
        ADDRESS=$(echo "$BTC_ADDRESS" | grep -oE 'tb1[a-z0-9]+' | head -1)
    fi

    if [ -n "$ADDRESS" ]; then
        echo -e "${GREEN}✓${NC} Bitcoin Address: ${BLUE}$ADDRESS${NC}"
        echo ""
        echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${YELLOW}📝 STEP 1: Fund this address with testnet BTC${NC}"
        echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo ""
        echo "  Address: $ADDRESS"
        echo ""
        echo "  Bitcoin Testnet Faucets:"
        echo "  • https://coinfaucet.eu/en/btc-testnet/"
        echo "  • https://testnet-faucet.mempool.co/"
        echo "  • https://bitcoinfaucet.uo1.net/"
        echo ""
        echo "  Recommended amount: 0.001 BTC (100,000 satoshis)"
        echo ""
        echo "  Block Explorer:"
        echo "  https://blockstream.info/testnet/address/$ADDRESS"
        echo ""
    else
        echo -e "${RED}✗${NC} Could not extract Bitcoin address from response"
        echo "$BTC_ADDRESS"
        exit 1
    fi
else
    echo -e "${RED}✗${NC} Failed to get Bitcoin address"
    echo "$BTC_ADDRESS"
    exit 1
fi

# Step 2: Wait for user to fund address
echo -e "${YELLOW}[2/5]${NC} Waiting for funding..."
echo ""
read -p "$(echo -e ${YELLOW}Press Enter after funding the address and waiting for 1 confirmation...${NC})"
echo ""

# Step 3: Check/Initialize configuration
echo -e "${YELLOW}[3/5]${NC} Checking canister configuration..."
CONFIG=$(dfx canister call $CANISTER_ID get_config --wallet $WALLET_ID --network $NETWORK 2>&1 || echo "")

if echo "$CONFIG" | grep -q "null" || [ -z "$CONFIG" ]; then
    echo -e "${YELLOW}⚠${NC}  No configuration found. Initializing..."
    echo ""

    # Initialize with Bitcoin-friendly config
    INIT_RESULT=$(dfx canister call $CANISTER_ID initialize '(
      record {
        name = "ChainGuard Bitcoin Test";
        default_threshold = record { required = 1; total = 1; };
        supported_chains = vec { "Sepolia"; "Bitcoin"; "BitcoinTestnet" };
        policies = vec {
          record {
            name = "Allow Small Bitcoin Transfers";
            conditions = vec {
              variant { MaxAmount = 100000 };
              variant { AllowedChains = vec { "BitcoinTestnet"; "Bitcoin" } };
            };
            action = variant { Allow };
            priority = 1;
          };
          record {
            name = "Require Threshold for Large Transfers";
            conditions = vec {
              variant { MaxAmount = 10000000 };
              variant { AllowedChains = vec { "BitcoinTestnet"; "Bitcoin" } };
            };
            action = variant { RequireThreshold = record { required = 2; from_roles = vec { variant { Owner }; variant { Operator } } } };
            priority = 2;
          }
        };
      }
    )' --wallet $WALLET_ID --network $NETWORK 2>&1)

    if echo "$INIT_RESULT" | grep -q "Ok"; then
        echo -e "${GREEN}✓${NC} Configuration initialized successfully"
    else
        echo -e "${RED}✗${NC} Failed to initialize configuration"
        echo "$INIT_RESULT"
    fi
else
    echo -e "${GREEN}✓${NC} Configuration already exists"
fi
echo ""

# Step 4: Execute test transfer
echo -e "${YELLOW}[4/5]${NC} Executing test Bitcoin transfer..."
echo ""

# Use a well-known testnet address for testing
TEST_RECIPIENT="tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx"
AMOUNT=10000  # 0.0001 BTC = 10,000 satoshis

echo -e "  From:      ${BLUE}$ADDRESS${NC}"
echo -e "  To:        ${BLUE}$TEST_RECIPIENT${NC}"
echo -e "  Amount:    ${BLUE}$AMOUNT satoshis (0.0001 BTC)${NC}"
echo -e "  Network:   ${BLUE}$BTC_NETWORK${NC}"
echo ""

RESULT=$(dfx canister call $CANISTER_ID request_action "(
  variant {
    BitcoinTransfer = record {
      to = \"$TEST_RECIPIENT\";
      amount = $AMOUNT;
      network = \"$BTC_NETWORK\"
    }
  }
)" --wallet $WALLET_ID --network $NETWORK 2>&1)

echo "Response:"
echo "$RESULT"
echo ""

# Parse result
if echo "$RESULT" | grep -q "Executed"; then
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}✓ Transaction executed successfully!${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    # Try to extract TX hash or message
    if echo "$RESULT" | grep -q "Transaction submitted"; then
        echo ""
        echo "  Status: Transaction submitted to Bitcoin network"
        echo "  Note: Transaction is being processed by Bitcoin canister"
        echo ""
        echo "  Monitor the address on block explorer:"
        echo "  https://blockstream.info/testnet/address/$ADDRESS"
    fi

elif echo "$RESULT" | grep -q "PendingSignatures"; then
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}⏳ Transaction requires threshold signatures${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    REQUEST_ID=$(echo "$RESULT" | grep -oP '(?<=id = )[0-9]+' || echo "unknown")
    echo "  Request ID: $REQUEST_ID"
    echo ""
    echo "  To approve, run:"
    echo "  dfx canister call $CANISTER_ID sign_request '($REQUEST_ID)' --wallet $WALLET_ID --network $NETWORK"

elif echo "$RESULT" | grep -q "Denied"; then
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${RED}✗ Transaction denied by policy${NC}"
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    REASON=$(echo "$RESULT" | grep -oP '(?<=reason = ")[^"]+' || echo "unknown")
    echo "  Reason: $REASON"

else
    echo -e "${RED}✗${NC} Unknown result or error occurred"
    echo "  Check the response above for details"
fi
echo ""

# Step 5: Check audit logs
echo -e "${YELLOW}[5/5]${NC} Checking audit logs..."
AUDIT=$(dfx canister call $CANISTER_ID get_audit_logs '(null, null)' --wallet $WALLET_ID --network $NETWORK 2>&1 || echo "")

if [ -n "$AUDIT" ]; then
    # Count entries
    ENTRY_COUNT=$(echo "$AUDIT" | grep -c "id = " || echo "0")
    echo -e "${GREEN}✓${NC} Found $ENTRY_COUNT audit entries"

    if [ "$ENTRY_COUNT" -gt 0 ]; then
        echo ""
        echo "Latest audit entry:"
        # Show just the most recent entry
        echo "$AUDIT" | grep -A 10 "action_type = " | head -15
    fi
else
    echo -e "${YELLOW}⚠${NC}  Could not retrieve audit logs"
fi

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✓ Bitcoin integration test completed!${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Summary
echo -e "${YELLOW}Summary:${NC}"
echo "  • Canister:        $CANISTER_ID"
echo "  • Wallet:          $WALLET_ID"
echo "  • Bitcoin Address: $ADDRESS"
echo "  • Test Recipient:  $TEST_RECIPIENT"
echo "  • Amount:          $AMOUNT satoshis"
echo "  • Network:         $BTC_NETWORK"
echo ""

echo -e "${YELLOW}Next Steps:${NC}"
echo "  1. Monitor transaction on block explorer:"
echo "     https://blockstream.info/testnet/address/$ADDRESS"
echo ""
echo "  2. Wait ~10-60 minutes for Bitcoin confirmation"
echo ""
echo "  3. Check audit logs:"
echo "     dfx canister call $CANISTER_ID get_audit_logs '(null, null)' \\"
echo "       --wallet $WALLET_ID --network $NETWORK"
echo ""
echo "  4. Try additional transfers or test threshold signatures"
echo ""
