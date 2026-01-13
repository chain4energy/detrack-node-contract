#!/bin/bash
# deploy_contract.sh - Script to build and deploy the DeTrack smart contract

# Load environment variables
source config/.env

echo "=== DeTrack Smart Contract Deployment ==="

# Step 1: Build the contract
echo -e "\n=== 1. Building Contract ==="
cargo build --release --target wasm32-unknown-unknown

if [ $? -ne 0 ]; then
  echo "Error: Contract build failed"
  exit 1
fi

# Step 2: Optimize the WASM file using CosmWasm rust-optimizer
echo -e "\n=== 2. Optimizing WASM File with rust-optimizer ==="

# Check if Docker is available
if ! command -v docker &> /dev/null; then
  echo "Error: Docker is not installed. Please install Docker to optimize WASM files."
  exit 1
fi

echo "Running CosmWasm rust-optimizer (this may take several minutes)..."
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="detrack-node-contract_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.16.0

if [ $? -ne 0 ]; then
  echo "Error: WASM optimization failed"
  exit 1
fi

# The optimizer creates the optimized file in artifacts/
# Note: rust-optimizer creates detrack_node_contract.wasm (based on Cargo.toml name)
OPTIMIZED_WASM="artifacts/detrack_node_contract.wasm"

if [ ! -f "$OPTIMIZED_WASM" ]; then
  echo "Error: Optimized WASM file not found at $OPTIMIZED_WASM"
  echo "Available files in artifacts/:"
  ls -lh artifacts/*.wasm 2>/dev/null || echo "No WASM files found"
  exit 1
fi

# Check file size (should be under 819KB limit)
FILE_SIZE=$(stat -f%z "$OPTIMIZED_WASM" 2>/dev/null || stat -c%s "$OPTIMIZED_WASM" 2>/dev/null)
FILE_SIZE_KB=$((FILE_SIZE / 1024))
echo "Optimized WASM file size: ${FILE_SIZE_KB}KB"

if [ $FILE_SIZE -gt 819200 ]; then
  echo "Warning: WASM file is ${FILE_SIZE_KB}KB, exceeds 800KB limit!"
  exit 1
fi

echo "WASM optimization successful!"

# Step 3: Store the contract on the blockchain
echo -e "\n=== 3. Storing Contract on Blockchain ==="
# Use absolute path for the WASM file
FULL_WASM_PATH=$(realpath "$OPTIMIZED_WASM")
echo "Using WASM file at: $FULL_WASM_PATH"
TX_RESULT=$(c4ed --home $HOME_DIR tx wasm store "$FULL_WASM_PATH" \
  --from "$ADMIN_NAME" \
  --node "$C4E_RPC_ENDPOINT" \
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
echo -e "\n=== 4. Getting Code ID ==="
CODE_QUERY=$(c4ed --home $HOME_DIR query tx "$TX_HASH" --node "$C4E_RPC_ENDPOINT" --chain-id "$C4E_CHAIN_ID" -o json)

if [ $? -ne 0 ]; then
  echo "Error: Failed to query transaction"
  exit 1
fi

CODE_ID=$(echo "$CODE_QUERY" | jq -r '.logs[0].events[] | select(.type=="store_code").attributes[] | select(.key=="code_id").value')

if [ -z "$CODE_ID" ] || [ "$CODE_ID" == "null" ]; then
  echo "Error: Failed to extract code ID"
  exit 1
fi

echo "Contract code ID: $CODE_ID"

ACTION=${1:-store} # Default to 'store' if no argument is provided
echo -e "\\n=== Action: $ACTION ==="

if [ "$ACTION" == "store" ]; then
    # Step 5: Instantiate the contract
    echo -e "\\n=== 5. Instantiating Contract ==="
    INIT_MSG='{"admin":"'$APP_ADMIN'","did_contract_address":"'$DID_CONTRACT_ADDRESS'","version":"'$DETRACK_SC_VERSION'","min_stake_tier1":"'$MIN_STAKE_TIER1'","min_stake_tier2":"'$MIN_STAKE_TIER2'","min_stake_tier3":"'$MIN_STAKE_TIER3'","deposit_tier1":"'$DEPOSIT_TIER1'","deposit_tier2":"'$DEPOSIT_TIER2'","deposit_tier3":"'$DEPOSIT_TIER3'","use_whitelist":'$USE_WHITELIST',"deposit_unlock_period_blocks":'$DEPOSIT_UNLOCK_PERIOD_BLOCKS',"max_batch_size":'${MAX_BATCH_SIZE:-100}'}'

    echo "Initialization message:"
    echo "$INIT_MSG"

    INIT_RESULT=$(c4ed --home $HOME_DIR tx wasm instantiate "$CODE_ID" "$INIT_MSG" \
      --label "$DETRACK_SC_LABEL" \
      --admin "$APP_ADMIN" \
      --from "$ADMIN_NAME" \
      --node "$C4E_RPC_ENDPOINT" \
      --chain-id "$C4E_CHAIN_ID" \
      --gas auto \
      --gas-adjustment 1.3 \
      --broadcast-mode sync \
      -y)

    echo "$INIT_RESULT"

    INIT_TX_HASH=$(echo "$INIT_RESULT" | grep -oP '(?<=txhash: ).*')
    if [ -z "$INIT_TX_HASH" ]; then
      echo "Error: Failed to extract instantiation transaction hash"
      exit 1
    fi

    echo "Instantiation transaction hash: $INIT_TX_HASH"
    echo "Waiting for transaction to be included in a block..."
    sleep 5

    # Step 6: Get the contract address
    echo -e "\\n=== 6. Getting Contract Address ==="
    INIT_QUERY=$(c4ed --home $HOME_DIR query tx "$INIT_TX_HASH" --node "$C4E_RPC_ENDPOINT" --chain-id "$C4E_CHAIN_ID" -o json)

    if [ $? -ne 0 ]; then
      echo "Error: Failed to query instantiation transaction"
      exit 1
    fi

    CONTRACT_ADDR=$(echo "$INIT_QUERY" | jq -r '.logs[0].events[] | select(.type=="instantiate").attributes[] | select(.key=="_contract_address").value')

    if [ -z "$CONTRACT_ADDR" ] || [ "$CONTRACT_ADDR" == "null" ]; then
      echo "Error: Failed to extract contract address"
      exit 1
    fi

    echo -e "\\n=== Contract Instantiation Successful ==="
    echo "New contract address: $CONTRACT_ADDR"
    echo "Code ID used for instantiation: $CODE_ID"

    # Update .env file with the new contract address
    echo -e "\\n=== Updating .env file with new contract address ==="
    if [ -f "config/.env" ]; then
      if grep -q "DETRACK_SMART_CONTRACT_ADDRESS=" "config/.env"; then
        sed -i "s|DETRACK_SMART_CONTRACT_ADDRESS=.*|DETRACK_SMART_CONTRACT_ADDRESS=$CONTRACT_ADDR|" "config/.env"
      else
        echo "DETRACK_SMART_CONTRACT_ADDRESS=$CONTRACT_ADDR" >> "config/.env"
      fi
      echo "Updated DETRACK_SMART_CONTRACT_ADDRESS in config/.env"
    else
      echo "Warning: config/.env file not found, unable to update contract address"
    fi
    
    FINAL_CONTRACT_ADDR="$CONTRACT_ADDR"

elif [ "$ACTION" == "migrate" ]; then
    echo -e "\\n=== 5. Migrating Contract ==="
    
    if ! grep -q "DETRACK_SMART_CONTRACT_ADDRESS=" "config/.env"; then
        echo "Error: DETRACK_SMART_CONTRACT_ADDRESS not found in config/.env. Cannot migrate."
        exit 1
    fi
    EXISTING_CONTRACT_ADDR=$(grep "DETRACK_SMART_CONTRACT_ADDRESS=" "config/.env" | cut -d '=' -f2)
    if [ -z "$EXISTING_CONTRACT_ADDR" ]; then
        echo "Error: DETRACK_SMART_CONTRACT_ADDRESS is empty in config/.env. Cannot migrate."
        exit 1
    fi
    echo "Migrating existing contract at address: $EXISTING_CONTRACT_ADDR"
    echo "Using new code ID for migration: $CODE_ID"

    #MIGRATE_MSG="{\\"new_version\\":\\"$DETRACK_SC_VERSION\\"}" # Corrected MIGRATE_MSG
    #MIGRATE_MSG="{\\\"new_version\\\":\\\"$DETRACK_SC_VERSION\\\"}"
    MIGRATE_MSG="{\"migrate\":{\"new_version\":\"$DETRACK_SC_VERSION\"}}"
    echo "Migration message:"
    echo "$MIGRATE_MSG"

    MIGRATE_RESULT=$(c4ed --home $HOME_DIR tx wasm migrate "$EXISTING_CONTRACT_ADDR" "$CODE_ID" "$MIGRATE_MSG" \
      --from "$ADMIN_NAME" \
      --node "$C4E_RPC_ENDPOINT" \
      --chain-id "$C4E_CHAIN_ID" \
      --gas auto \
      --gas-adjustment 1.3 \
      --broadcast-mode sync \
      -y)
    
    echo "$MIGRATE_RESULT"
    MIGRATE_TX_HASH=$(echo "$MIGRATE_RESULT" | grep -oP '(?<=txhash: ).*')
    if [ -z "$MIGRATE_TX_HASH" ]; then
      echo "Error: Failed to extract migration transaction hash. Migration might have failed."
      if echo "$MIGRATE_RESULT" | grep -q -i "error"; then
          echo "Migration command failed. Please check the output above for details."
      fi
      exit 1
    fi
    echo "Migration transaction hash: $MIGRATE_TX_HASH"
    echo "Waiting for migration transaction to be included in a block..."
    sleep 5

    echo "Verifying migration..."
    QUERY_CONFIG_MSG='{"config":{}}'
    UPDATED_CONFIG_QUERY_RESULT=$(c4ed --home $HOME_DIR query wasm contract-state smart "$EXISTING_CONTRACT_ADDR" "$QUERY_CONFIG_MSG" --node "$C4E_RPC_ENDPOINT" --chain-id "$C4E_CHAIN_ID" -o json 2>&1)
    
    if [ $? -ne 0 ]; then
        echo "Error querying contract config after migration: $UPDATED_CONFIG_QUERY_RESULT"
        echo "Migration transaction was sent, but verification of new config failed."
    else
        echo "Updated contract config query result: $UPDATED_CONFIG_QUERY_RESULT"
        ACTUAL_NEW_VERSION=$(echo "$UPDATED_CONFIG_QUERY_RESULT" | jq -r '.data.version') 
        
        if [ "$ACTUAL_NEW_VERSION" == "$DETRACK_SC_VERSION" ]; then
            echo "Contract migration successful. Version updated to $ACTUAL_NEW_VERSION."
        else
            echo "Warning: Contract migration transaction submitted, but version verification failed or did not match."
            echo "Expected version: $DETRACK_SC_VERSION, Got from query: $ACTUAL_NEW_VERSION (using path .data.version)"
            echo "Please verify the contract state manually."
        fi
    fi
    FINAL_CONTRACT_ADDR="$EXISTING_CONTRACT_ADDR"
else
    echo "Error: Invalid action '$ACTION'. Usage: $0 [store|migrate]"
    exit 1
fi

echo -e "\\n=== Script Complete ==="
if [ -n "$FINAL_CONTRACT_ADDR" ]; then
  echo "You can interact with the contract at address: $FINAL_CONTRACT_ADDR"
  if [ "$ACTION" == "migrate" ]; then
    echo "(This is the existing address that was migrated)"
  fi
else
  if [ "$ACTION" == "store" ]; then
    echo "Contract instantiation may have failed to retrieve the address."
  fi
fi
