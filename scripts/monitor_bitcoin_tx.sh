#!/bin/bash

# Monitor Bitcoin transaction propagation
# Usage: ./scripts/monitor_bitcoin_tx.sh TXID

TXID=$1
NETWORK="testnet"

if [ -z "$TXID" ]; then
    echo "❌ Error: TXID required"
    echo "Usage: $0 <txid>"
    exit 1
fi

echo "🔍 Monitoring Bitcoin transaction: $TXID"
echo "🌐 Network: $NETWORK"
echo "⏱️  Checking every 30 seconds..."
echo ""

ATTEMPTS=0
MAX_ATTEMPTS=60  # 30 minutes total

while [ $ATTEMPTS -lt $MAX_ATTEMPTS ]; do
    ATTEMPTS=$((ATTEMPTS + 1))
    ELAPSED=$((ATTEMPTS * 30))

    echo "[Attempt $ATTEMPTS/$MAX_ATTEMPTS - ${ELAPSED}s elapsed]"

    # Check Blockstream API
    RESPONSE=$(curl -s "https://blockstream.info/$NETWORK/api/tx/$TXID" 2>&1)

    if echo "$RESPONSE" | grep -q "Transaction not found"; then
        echo "⏳ Not found yet - waiting..."
    elif echo "$RESPONSE" | grep -q "txid"; then
        echo "✅ TRANSACTION FOUND!"
        echo ""
        echo "📊 Transaction details:"
        echo "$RESPONSE" | jq '{txid, status: .status.confirmed, block_height: .status.block_height}'
        echo ""
        echo "🔗 View on Blockstream:"
        echo "   https://blockstream.info/$NETWORK/tx/$TXID"
        echo ""
        echo "🔗 View on Mempool.space:"
        echo "   https://mempool.space/$NETWORK/tx/$TXID"
        exit 0
    else
        echo "⚠️  API response: $RESPONSE"
    fi

    if [ $ATTEMPTS -lt $MAX_ATTEMPTS ]; then
        sleep 30
    fi
done

echo ""
echo "⏰ Timeout reached after 30 minutes"
echo "💡 Transaction may still propagate - check manually:"
echo "   https://blockstream.info/$NETWORK/tx/$TXID"
exit 1
