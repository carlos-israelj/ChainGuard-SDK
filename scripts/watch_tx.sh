#!/bin/bash

# Monitor Bitcoin transaction on testnet4
# Usage: ./scripts/watch_tx.sh TXID

TXID=${1:-"b8656cc2f7dbc54c0a37b4b02557642b7e5d658cc4495e43b0b21bf65189c81d"}
NETWORK="testnet4"
CHECK_INTERVAL=30  # seconds
MAX_CHECKS=60      # 30 minutes total

echo "🔍 Monitoring Bitcoin transaction on $NETWORK"
echo "📝 TXID: $TXID"
echo "⏱️  Checking every $CHECK_INTERVAL seconds (max $MAX_CHECKS checks)"
echo ""
echo "🔗 URLs to check manually:"
echo "   https://mempool.space/$NETWORK/tx/$TXID"
echo "   https://blockstream.info/$NETWORK/tx/$TXID"
echo ""

for i in $(seq 1 $MAX_CHECKS); do
    ELAPSED=$((i * CHECK_INTERVAL))
    MINUTES=$((ELAPSED / 60))

    echo "[Check $i/$MAX_CHECKS - ${MINUTES}m ${ELAPSED}s elapsed]"

    # Check mempool.space API
    RESPONSE=$(curl -s "https://mempool.space/$NETWORK/api/tx/$TXID" 2>&1)

    if echo "$RESPONSE" | grep -q "Transaction not found"; then
        echo "⏳ Not found yet - waiting $CHECK_INTERVAL seconds..."
    elif echo "$RESPONSE" | grep -q "txid"; then
        echo ""
        echo "✅ TRANSACTION FOUND!"
        echo ""
        echo "📊 Transaction details:"
        echo "$RESPONSE" | jq '{txid, status, fee, size, weight, vsize}' 2>/dev/null || echo "$RESPONSE"
        echo ""
        echo "🔗 View on mempool.space:"
        echo "   https://mempool.space/$NETWORK/tx/$TXID"
        echo ""
        echo "🔗 View on Blockstream:"
        echo "   https://blockstream.info/$NETWORK/tx/$TXID"
        echo ""
        exit 0
    else
        echo "⚠️  API response: $RESPONSE"
    fi

    if [ $i -lt $MAX_CHECKS ]; then
        sleep $CHECK_INTERVAL
    fi
done

echo ""
echo "⏰ Timeout reached after 30 minutes"
echo "💡 Transaction may still propagate - check manually:"
echo "   https://mempool.space/$NETWORK/tx/$TXID"
echo "   https://blockstream.info/$NETWORK/tx/$TXID"
echo ""
echo "Note: ICP Bitcoin propagation can take 5-15 minutes depending on network conditions"
exit 1
