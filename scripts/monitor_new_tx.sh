#!/bin/bash

TXID="645911f1bfdebd6091513cb741e35326c4a837994dc01ba7dc356b93bdd5c43b"
MAX_CHECKS=30  # 30 checks = 15 minutes at 30 second intervals
CHECK_INTERVAL=30

echo "=== Monitoring Transaction ==="
echo "TXID: $TXID"
echo "URL: https://mempool.space/testnet4/tx/$TXID"
echo "Started at: $(date)"
echo "Will check every $CHECK_INTERVAL seconds for $MAX_CHECKS times"
echo ""

for i in $(seq 1 $MAX_CHECKS); do
    echo "[Check $i/$MAX_CHECKS at $(date)]"

    RESULT=$(curl -s "https://mempool.space/testnet4/api/tx/$TXID" 2>&1)

    if echo "$RESULT" | grep -q "Transaction not found"; then
        echo "❌ Not found yet"
    elif echo "$RESULT" | grep -q '"txid"'; then
        echo "✅ TRANSACTION FOUND!"
        echo "$RESULT" | jq '.' 2>/dev/null || echo "$RESULT"
        exit 0
    else
        echo "⚠️  Unknown response: $RESULT"
    fi

    if [ $i -lt $MAX_CHECKS ]; then
        sleep $CHECK_INTERVAL
    fi
done

echo ""
echo "=== Monitoring Complete ==="
echo "Transaction still not found after $MAX_CHECKS checks"
echo "This may indicate a propagation issue with the Bitcoin Canister"
