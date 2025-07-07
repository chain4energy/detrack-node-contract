#!/bin/bash
# fund_detrack_wallet.sh - Sends tokens to DeTrack wallet for testing

# Load environment variables
source ./config/.env

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=== Funding DeTrack Wallet ===${NC}"

if [ -z "$DETRACK1_NODE" ]; then
  echo -e "${RED}Error: DETRACK1_NODE environment variable is not set${NC}"
  exit 1
fi

if [ -z "$ALICE_NAME" ] || [ -z "$ALICE" ]; then
  echo -e "${RED}Error: ALICE_NAME or ALICE environment variable is not set${NC}"
  exit 1
fi

echo -e "${YELLOW}Sending funds from $ALICE to $DETRACK1_NODE...${NC}"

# Use c4e-chain CLI to send funds
# Adjust the path to your specific c4e-chain CLI location
C4E_BIN="${HOME_DIR}/bin/c4ed"

# Send 100000000uc4e (100 C4E) tokens from alice to the DeTrack wallet
TX_RESULT=$(${C4E_BIN} tx bank send $ALICE_NAME $DETRACK1_NODE 100000000uc4e \
  --chain-id=${C4E_CHAIN_ID} \
  --gas=auto \
  --gas-adjustment=1.3 \
  --broadcast-mode=block \
  --keyring-backend=${KEYRING_BACKEND} \
  --yes \
  -o json)

# Check if transfer was successful
TX_HASH=$(echo $TX_RESULT | jq -r '.txhash')
CODE=$(echo $TX_RESULT | jq -r '.code')

if [ "$CODE" = "0" ]; then
  echo -e "${GREEN}Transaction successful!${NC}"
  echo -e "Transaction hash: ${GREEN}$TX_HASH${NC}"
  
  # Check the balance of the DeTrack wallet
  echo -e "\n${YELLOW}Checking DETRACK1_NODE balance...${NC}"
  BALANCE=$(${C4E_BIN} query bank balances $DETRACK1_NODE --chain-id=${C4E_CHAIN_ID} -o json | jq -r '.balances[] | select(.denom=="uc4e") | .amount')
  
  if [ -n "$BALANCE" ]; then
    C4E_BALANCE=$(echo "scale=6; $BALANCE / 1000000" | bc)
    echo -e "${GREEN}Balance of $DETRACK1_NODE: $C4E_BALANCE C4E ($BALANCE uc4e)${NC}"
  else
    echo -e "${RED}Failed to retrieve balance${NC}"
  fi
else
  echo -e "${RED}Transaction failed with code: $CODE${NC}"
  echo "Error details:"
  echo $TX_RESULT | jq
fi
