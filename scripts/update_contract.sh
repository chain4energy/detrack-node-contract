#!/bin/bash
# update_contract.sh - Script to migrate an existing DeTrack contract to a new version

# Load environment variables
source config/.env

if [ -z "$DETRACK_SMART_CONTRACT_ADDRESS" ]; then
  echo "Error: DETRACK_SMART_CONTRACT_ADDRESS not set in config/.env"
  exit 1
fi

echo "=== DeTrack Smart Contract Update ==="
echo "Target contract address: $DETRACK_SMART_CONTRACT_ADDRESS"

# Step 1: Build the contract
echo -e "\n=== 1. Building Contract ==="
cd contracts/detrack-contract
cargo build --release --target wasm32-unknown-unknown

if [ $? -ne 0 ]; then
  echo "Error: Contract build failed"
  exit 1
fi

# Step 2: Optimize the WASM file
echo -e "\n=== 2. Optimizing WASM File ==="
mkdir -p artifacts
WASM_FILE="target/wasm32-unknown-unknown/release/detrack_contract.wasm"
OPTIMIZED_WASM="artifacts/detrack_contract_optimized.wasm"

# Check if the WASM file exists
if [ ! -f "$WASM_FILE" ]; then
  echo "Error: WASM file not found at $WASM_FILE"
  exit 1
fi

# Copy to artifacts
cp "$WASM_FILE" "$OPTIMIZED_WASM"
echo "WASM file copied to $OPTIMIZED_WASM"

# Step 3: Store the contract on the blockchain
echo -e "\n=== 3. Storing Updated Contract on Blockchain ==="
FULL_WASM_PATH=$(realpath "$OPTIMIZED_WASM")
echo "Using WASM file at: $FULL_WASM_PATH"
cd ../..
TX_RESULT=$(c4ed --home $HOME_DIR tx wasm store "$FULL_WASM_PATH" \
  --from "$ADMIN_NAME" \
  --chain-id "$C4E_CHAIN_ID" \
  --gas auto \
  --gas-adjustment 1.3 \
  --broadcast-mode sync \
  -y)

echo "$TX_RESULT"

# Extract the transaction hash from the result
TX_HASH=$(echo "$TX_RESULT" | grep -oP '(?<=txhash: ).*')
if [ -z "$TX_HASH" ]; then
  echo "Error: Failed to extract transaction hash"
  exit 1
fi

echo "Transaction hash: $TX_HASH"
echo "Waiting for transaction to be included in a block..."
sleep 5

# Step 4: Get the code ID
echo -e "\n=== 4. Getting New Code ID ==="
CODE_QUERY=$(c4ed --home $HOME_DIR query tx "$TX_HASH" --chain-id "$C4E_CHAIN_ID" -o json)

if [ $? -ne 0 ]; then
  echo "Error: Failed to query transaction"
  exit 1
fi

CODE_ID=$(echo "$CODE_QUERY" | jq -r '.logs[0].events[] | select(.type=="store_code").attributes[] | select(.key=="code_id").value')

if [ -z "$CODE_ID" ] || [ "$CODE_ID" == "null" ]; then
  echo "Error: Failed to extract code ID"
  exit 1
fi

echo "New contract code ID: $CODE_ID"

# Step 5: Migrate the contract to the new code ID
echo -e "\n=== 5. Migrating Contract to New Code ID ==="
MIGRATE_MSG="{\"migrate\":{\"new_version\":\"0.2.0\"}}"

echo "Migration message:"
echo "$MIGRATE_MSG"

MIGRATE_RESULT=$(c4ed --home $HOME_DIR tx wasm migrate "$DETRACK_SMART_CONTRACT_ADDRESS" "$CODE_ID" "$MIGRATE_MSG" \
  --from "$ADMIN_NAME" \
  --chain-id "$C4E_CHAIN_ID" \
  --gas auto \
  --gas-adjustment 1.3 \
  --broadcast-mode sync \
  -y)

echo "$MIGRATE_RESULT"

MIGRATE_TX_HASH=$(echo "$MIGRATE_RESULT" | grep -oP '(?<=txhash: ).*')
if [ -z "$MIGRATE_TX_HASH" ]; then
  echo "Error: Failed to extract migration transaction hash"
  exit 1
fi

echo "Migration transaction hash: $MIGRATE_TX_HASH"
echo "Waiting for transaction to be included in a block..."
sleep 5

# Step 6: Verify the contract migration
echo -e "\n=== 6. Verifying Contract Migration ==="
CONFIG_QUERY=$(c4ed --home $HOME_DIR query wasm contract-state smart "$DETRACK_SMART_CONTRACT_ADDRESS" '{"config":{}}' --chain-id "$C4E_CHAIN_ID" -o json)

if [ $? -ne 0 ]; then
  echo "Error: Failed to query contract config"
  exit 1
fi

CONTRACT_VERSION=$(echo "$CONFIG_QUERY" | jq -r '.data.version')

if [ -z "$CONTRACT_VERSION" ] || [ "$CONTRACT_VERSION" == "null" ]; then
  echo "Error: Failed to extract contract version"
  exit 1
fi

echo -e "\n=== Contract Update Successful ==="
echo "Contract address: $DETRACK_SMART_CONTRACT_ADDRESS"
echo "New code ID: $CODE_ID"
echo "Contract version: $CONTRACT_VERSION"

echo -e "\n=== Testing NodeInfo Query ==="
NODE_INFO_QUERY=$(c4ed --home $HOME_DIR query wasm contract-state smart "$DETRACK_SMART_CONTRACT_ADDRESS" \
  '{"node_info":{"address":"'$DETRACK1_NODE'"}}' --chain-id "$C4E_CHAIN_ID" -o json)

echo "NodeInfo Query Result:"
echo "$NODE_INFO_QUERY" | jq

echo -e "\n=== Update Complete ==="
