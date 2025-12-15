#!/bin/bash

TXID="acf3707ca8b7c05d1c25222ea8eeec6ee7b94ced5f278cd343198f99c287b58c"
MAX_CHECKS=30
CHECK_INTERVAL=30

echo "=== Monitoring NEW Transaction (with DFINITY pattern) ==="
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
echo "=== Monitoring Complete ===\"
echo "Transaction still not found after $MAX_CHECKS checks"
