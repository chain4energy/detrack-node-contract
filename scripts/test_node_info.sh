#!/bin/bash
# test_node_info.sh - Script to test NodeInfo functionality

# Load environment variables
source config/.env

echo "=== Testing NodeInfo Query ==="
echo "Contract Address: $DETRACK_SMART_CONTRACT_ADDRESS"
echo "Node Address: $DETRACK1_NODE"

# Test the NodeInfo query
echo -e "\n=== Running NodeInfo Query ==="

NODE_INFO_CMD='{
  "node_info": {
    "address": "'$DETRACK1_NODE'"
  }
}'

echo "Node info query command:"
echo "$NODE_INFO_CMD"

c4ed --home $HOME_DIR query wasm contract-state smart $DETRACK_SMART_CONTRACT_ADDRESS \
  "$NODE_INFO_CMD" \
  --node "$C4E_RPC_ENDPOINT" \
  --chain-id "$C4E_CHAIN_ID"
# Test adding reputation to the node via admin
echo -e "\n=== Testing Admin Update Node Reputation ==="

UPDATE_REP_CMD='{
  "admin": {
    "update_node_reputation": {
      "node_address": "'$DETRACK1_NODE'",
      "reputation": "75"
    }
  }
}'

echo "Update reputation command:"
echo "$UPDATE_REP_CMD"

c4ed --home $HOME_DIR tx wasm execute $DETRACK_SMART_CONTRACT_ADDRESS \
  "$UPDATE_REP_CMD" \
  --from $ADMIN_NAME \
  --node "$C4E_RPC_ENDPOINT" \
  --chain-id $C4E_CHAIN_ID \
  --gas auto \
  --gas-adjustment 1.3 \
  --broadcast-mode sync \
  -y

echo "Waiting for transaction to be processed..."
sleep 5

# Check if reputation was updated
echo -e "\n=== Checking Updated Reputation ==="
c4ed --home $HOME_DIR query wasm contract-state smart $DETRACK_SMART_CONTRACT_ADDRESS \
  "$NODE_INFO_CMD" \
  --node "$C4E_RPC_ENDPOINT" \
  --chain-id "$C4E_CHAIN_ID"

echo -e "\n=== Testing Complete ==="
