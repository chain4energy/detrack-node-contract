#!/bin/bash
# test-contract.sh - Script to test DeTrack Smart Contract functionality

# Load environment variables
source config/.env

echo "=== DeTrack Smart Contract Testing ==="
echo "Contract Address: $DETRACK_SMART_CONTRACT_ADDRESS"
echo "Node Address: $DETRACK1_NODE"

# Generate a unique data hash for testing
TIMESTAMP=$(date +%s)
TEST_DATA_HASH="test_hash_${TIMESTAMP}"

echo -e "\n=== 1. Testing StoreProof ==="

# Simple direct approach - we'll write the JSON command in a single line with minimal escaping
STORE_CMD='{
  "node": {
    "store_proof": {
      "data_hash": "'$TEST_DATA_HASH'",
      "original_data_reference": "https://example.com/data/'$TIMESTAMP'",
      "data_owner": "'$DETRACK1_NODE'",
      "metadata_json": "{\"timestamp\":\"'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'\",\"facility_id\":\"F123\",\"device_id\":\"D456\"}"
    }
  }
}'

# For debugging
echo "Command to execute:"
echo "$STORE_CMD"

# Execute the command
c4ed --home $HOME_DIR tx wasm execute $DETRACK_SMART_CONTRACT_ADDRESS \
  "$STORE_CMD" \
  --from $DETRACK1_NODE_NAME \
  --chain-id $C4E_CHAIN_ID \
  --gas auto \
  --gas-adjustment 1.3 \
  --broadcast-mode sync \
  -y

echo "Waiting for transaction to be processed..."
sleep 5

# Query contract to check if proof exists
echo -e "\n=== 2. Testing Query ProofByHash ==="

QUERY_CMD='{
  "proof_by_hash": {
    "data_hash": "'$TEST_DATA_HASH'"
  }
}'

echo "Query command:"
echo "$QUERY_CMD"

c4ed --home $HOME_DIR query wasm contract-state smart $DETRACK_SMART_CONTRACT_ADDRESS \
  "$QUERY_CMD" \
  --chain-id $C4E_CHAIN_ID

# Test verifying the proof
echo -e "\n=== 3. Testing VerifyProof ==="

VERIFY_CMD='{
  "node": {
    "verify_proof": {
      "data_hash": "'$TEST_DATA_HASH'"
    }
  }
}'

echo "Verify command:"
echo "$VERIFY_CMD"

c4ed --home $HOME_DIR tx wasm execute $DETRACK_SMART_CONTRACT_ADDRESS \
  "$VERIFY_CMD" \
  --from $DETRACK1_NODE_NAME \
  --chain-id $C4E_CHAIN_ID \
  --gas auto \
  --gas-adjustment 1.3 \
  --broadcast-mode sync \
  -y

echo "Waiting for transaction to be processed..."
sleep 5

# Query Config
echo -e "\n=== 4. Testing Query Config ==="

CONFIG_CMD='{
  "config": {}
}'

echo "Config query command:"
echo "$CONFIG_CMD"

c4ed --home $HOME_DIR query wasm contract-state smart $DETRACK_SMART_CONTRACT_ADDRESS \
  "$CONFIG_CMD" \
  --chain-id $C4E_CHAIN_ID



# Query User
echo -e "\n=== 6. Testing Query User ==="

USER_QUERY_CMD='{
  "user": {
    "address": "'$DETRACK1_NODE'"
  }
}'

echo "User query command:"
echo "$USER_QUERY_CMD"

c4ed --home $HOME_DIR query wasm contract-state smart $DETRACK_SMART_CONTRACT_ADDRESS \
  "$USER_QUERY_CMD" \
  --chain-id $C4E_CHAIN_ID

# Test the NodeInfo query
echo -e "\n=== 7. Testing Query NodeInfo ==="

NODE_INFO_CMD='{
  "node_info": {
    "address": "'$DETRACK1_NODE'"
  }
}'

echo "Node info query command:"
echo "$NODE_INFO_CMD"

c4ed --home $HOME_DIR query wasm contract-state smart $DETRACK_SMART_CONTRACT_ADDRESS \
  "$NODE_INFO_CMD" \
  --chain-id $C4E_CHAIN_ID

echo -e "\n=== Testing Complete ==="
echo "Review the outputs above to verify contract functionality."
