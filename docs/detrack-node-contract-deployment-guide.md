# DeTrack Node Contract Deployment Guide

## Overview

This guide provides comprehensive instructions for deploying and operating the DeTrack Node Contract on Chain4Energy networks. The contract implements a decentralized node management and proof verification system for energy data.

## Prerequisites

### System Requirements
- **Rust**: 1.70+ with `wasm32-unknown-unknown` target
- **Docker**: For WASM optimization
- **c4ed CLI**: Chain4Energy command-line interface
- **Git**: For source code management
- **Make**: For build automation (optional)

### Development Tools

```bash
# Install Rust and WASM target
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Install Docker (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install docker.io
sudo usermod -aG docker $USER

# Install c4ed CLI
wget https://github.com/chain4energy/c4e-chain/releases/latest/download/c4ed-linux-amd64
chmod +x c4ed-linux-amd64
sudo mv c4ed-linux-amd64 /usr/local/bin/c4ed

# Verify installations
rustc --version
cargo --version
docker --version
c4ed version
```

## Network Configurations

### Mainnet (Perun)
- **Chain ID**: `perun-1`
- **RPC Endpoint**: `https://rpc.c4e.io:443`
- **REST Endpoint**: `https://lcd.c4e.io:443`
- **Gas Prices**: `0.0025uc4e`
- **Block Time**: ~6 seconds
- **Unbonding Period**: 100,800 blocks (~7 days)

### Testnet (Babajaga)
- **Chain ID**: `babajaga-1`
- **RPC Endpoint**: `https://rpc.babajaga.c4e.io:443`
- **REST Endpoint**: `https://lcd.babajaga.c4e.io:443`
- **Gas Prices**: `0.0025uc4e`
- **Faucet**: `https://faucet.babajaga.c4e.io`
- **Block Time**: ~6 seconds

### Local Development
- **Chain ID**: `c4e-dev`
- **RPC Endpoint**: `http://localhost:26657`
- **REST Endpoint**: `http://localhost:1317`
- **Gas Prices**: `0uc4e`

## Build Process

### 1. Clone Repository

```bash
git clone https://github.com/chain4energy/detrack-node-contract
cd detrack-node-contract
```

### 2. Development Build

```bash
# Build for development and testing
cargo build

# Run tests
cargo test

# Run specific test
cargo test test_register_node -- --nocapture

# Check linting
cargo clippy -- -D warnings

# Format code
cargo fmt
```

### 3. Production Build

#### Manual WASM Build

```bash
# Build WASM binary
cargo build --release --target wasm32-unknown-unknown

# Verify binary
ls -lh target/wasm32-unknown-unknown/release/detrack_node_contract.wasm
```

#### Optimized Build (Recommended)

```bash
# Using rust-optimizer Docker image (recommended for production)
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.15.0

# Or use cosmwasm/workspace-optimizer for multi-contract workspaces
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.15.0
```

### 4. Verify Build

```bash
# Check artifact size (should be < 2MB)
ls -lh artifacts/detrack_node_contract.wasm

# Verify checksums
cat artifacts/checksums.txt

# Validate WASM (if cosmwasm-check is available)
cosmwasm-check artifacts/detrack_node_contract.wasm
```

## Deployment Steps

### Step 1: Configure Network

#### Mainnet Configuration

```bash
c4ed config chain-id perun-1
c4ed config node https://rpc.c4e.io:443
c4ed config broadcast-mode block
c4ed config gas-prices 0.0025uc4e
```

#### Testnet Configuration

```bash
c4ed config chain-id babajaga-1
c4ed config node https://rpc.babajaga.c4e.io:443
c4ed config broadcast-mode block
c4ed config gas-prices 0.0025uc4e
```

### Step 2: Prepare Wallet

#### Import Existing Wallet

```bash
c4ed keys add deployer --recover
# Enter your mnemonic phrase when prompted
```

#### Create New Wallet

```bash
c4ed keys add deployer
# IMPORTANT: Save the mnemonic phrase securely!
```

#### Check Balance

```bash
DEPLOYER_ADDRESS=$(c4ed keys show deployer -a)
c4ed query bank balances $DEPLOYER_ADDRESS
```

### Step 3: Store Contract Code

```bash
# Store the optimized WASM file
c4ed tx wasm store artifacts/detrack_node_contract.wasm \
  --from deployer \
  --gas auto \
  --gas-adjustment 1.3 \
  --broadcast-mode block

# Extract CODE_ID from transaction output
# Look for "store_code" event with "code_id" attribute
export CODE_ID=<code_id_from_response>

# Alternative: Query latest code
c4ed query wasm list-code --reverse | head -20
```

#### Verify Code Storage

```bash
c4ed query wasm code $CODE_ID

# Download and compare hash
c4ed query wasm code $CODE_ID download.wasm
sha256sum download.wasm
sha256sum artifacts/detrack_node_contract.wasm
```

### Step 4: Instantiate Contract

#### Prepare Instantiation Message

Create `instantiate.json`:

```json
{
  "admin": null,
  "version": "v0.3.2",
  "did_contract_address": "c4e14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s86dt7n",
  "min_stake_tier1": "1000000000",
  "min_stake_tier2": "5000000000",
  "min_stake_tier3": "10000000000",
  "deposit_tier1": "100000000",
  "deposit_tier2": "200000000",
  "deposit_tier3": "300000000",
  "use_whitelist": false,
  "deposit_unlock_period_blocks": 100800,
  "max_batch_size": 100
}
```

**Parameter Explanation**:
- `min_stake_tier1`: 1 C4E (1,000,000,000 uc4e)
- `min_stake_tier2`: 5 C4E (5,000,000,000 uc4e)
- `min_stake_tier3`: 10 C4E (10,000,000,000 uc4e)
- `deposit_tier1`: 0.1 C4E (100,000,000 uc4e)
- `deposit_tier2`: 0.2 C4E (200,000,000 uc4e)
- `deposit_tier3`: 0.3 C4E (300,000,000 uc4e)
- `deposit_unlock_period_blocks`: 100,800 blocks (~7 days at 6s/block)
- `max_batch_size`: 100 batches max per proof (prevents resource strain)

#### Execute Instantiation

```bash
# Instantiate contract
c4ed tx wasm instantiate $CODE_ID "$(cat instantiate.json)" \
  --from deployer \
  --label "detrack-node-v0.1.0" \
  --gas auto \
  --gas-adjustment 1.3 \
  --no-admin \
  --broadcast-mode sync

# Extract contract address from output
export CONTRACT_ADDRESS=<contract_address_from_response>

# Save to environment
echo "export CONTRACT_ADDRESS=$CONTRACT_ADDRESS" >> ~/.bashrc
```

### Step 5: Verify Deployment

#### Check Contract Info

```bash
c4ed query wasm contract $CONTRACT_ADDRESS
c4ed query wasm contract-history $CONTRACT_ADDRESS
```

#### Test Configuration Query

```bash
c4ed query wasm contract-state smart $CONTRACT_ADDRESS '{
  "config": {}
}' --output json | jq
```

Expected response:
```json
{
  "admin": "c4e1deployer...",
  "version": "v0.3.2",
  "proof_count": 0,
  "min_reputation_threshold": 0,
  "treasury": null,
  "did_contract_address": "c4e14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s86dt7n",
  "min_stake_tier1": "1000000000",
  "min_stake_tier2": "5000000000",
  "min_stake_tier3": "10000000000",
  "deposit_tier1": "100000000",
  "deposit_tier2": "200000000",
  "deposit_tier3": "300000000",
  "use_whitelist": false,
  "deposit_unlock_period_blocks": 100800,
  "max_batch_size": 100
}
```
  "deposit_unlock_period_blocks": 100800
}
```

#### Test Basic Functionality

```bash
# 1. Check if deployer is registered as node (should be false initially)
c4ed query wasm contract-state smart $CONTRACT_ADDRESS '{
  "is_whitelisted": {
    "address": "'$DEPLOYER_ADDRESS'"
  }
}'

# 2. Check node info (should show not registered)
c4ed query wasm contract-state smart $CONTRACT_ADDRESS '{
  "node_info": {
    "address": "'$DEPLOYER_ADDRESS'"
  }
}'
```

## Environment-Specific Deployments

### Local Development Network

#### 1. Start Local Chain

```bash
# Initialize chain
c4ed init test-node --chain-id c4e-dev

# Add validator account
c4ed keys add validator
VALIDATOR_ADDR=$(c4ed keys show validator -a)

# Add genesis account with tokens
c4ed add-genesis-account $VALIDATOR_ADDR 1000000000000uc4e

# Create genesis transaction
c4ed gentx validator 1000000000uc4e \
  --chain-id c4e-dev \
  --moniker validator-node

# Collect genesis transactions
c4ed collect-gentxs

# Start chain
c4ed start
```

#### 2. Deploy Contract to Local Chain

```bash
# In a new terminal
export NODE="http://localhost:26657"

# Store contract
c4ed tx wasm store artifacts/detrack_node_contract.wasm \
  --from validator \
  --gas auto \
  --node $NODE \
  --chain-id c4e-dev \
  --keyring-backend test \
  --yes

# Wait for block
sleep 6

# Get CODE_ID
CODE_ID=$(c4ed query wasm list-code --node $NODE --output json | jq -r '.code_infos[-1].code_id')

# Instantiate
c4ed tx wasm instantiate $CODE_ID "$(cat instantiate.json)" \
  --from validator \
  --label "detrack-local" \
  --node $NODE \
  --chain-id c4e-dev \
  --keyring-backend test \
  --no-admin \
  --yes

# Get contract address
sleep 6
CONTRACT_ADDRESS=$(c4ed query wasm list-contract-by-code $CODE_ID --node $NODE --output json | jq -r '.contracts[-1]')
```

### Testnet (Babajaga) Deployment

#### 1. Get Testnet Tokens

```bash
# Request tokens from faucet
curl -X POST "https://faucet.babajaga.c4e.io/claim" \
  -H "Content-Type: application/json" \
  -d '{"address": "'$DEPLOYER_ADDRESS'"}'

# Verify balance
c4ed query bank balances $DEPLOYER_ADDRESS
```

#### 2. Deploy with Testnet Configuration

```bash
# Set testnet config
c4ed config chain-id babajaga-1
c4ed config node https://rpc.babajaga.c4e.io:443

# Store contract
c4ed tx wasm store artifacts/detrack_node_contract.wasm \
  --from deployer \
  --gas 2500000 \
  --gas-prices 0.0025uc4e \
  --broadcast-mode block

# Instantiate with testnet-specific parameters
c4ed tx wasm instantiate $CODE_ID "$(cat instantiate_testnet.json)" \
  --from deployer \
  --label "detrack-testnet-$(date +%s)" \
  --gas 1500000 \
  --gas-prices 0.0025uc4e \
  --no-admin \
  --broadcast-mode block
```

### Mainnet Production Deployment

#### 1. Pre-Deployment Checklist

- [ ] **Security Audit**: Contract audited by professional auditors
- [ ] **Testing**: All tests passing with >80% coverage
- [ ] **Testnet Validation**: Deployed and tested on testnet for 1+ week
- [ ] **Gas Optimization**: Gas costs measured and optimized
- [ ] **Wallet Security**: Deployer wallet secured with hardware device (Ledger)
- [ ] **Admin Controls**: Admin address reviewed and approved
- [ ] **Backup Procedures**: State backup scripts tested
- [ ] **Emergency Plan**: Documented response procedures
- [ ] **Documentation**: API docs and integration guides complete
- [ ] **Monitoring**: Alerting and monitoring systems ready

#### 2. Mainnet Deployment Steps

```bash
# Configure mainnet
c4ed config chain-id perun-1
c4ed config node https://rpc.c4e.io:443

# Verify deployer balance (should have sufficient C4E)
c4ed query bank balances $DEPLOYER_ADDRESS

# Store contract with higher gas limit
c4ed tx wasm store artifacts/detrack_node_contract.wasm \
  --from deployer \
  --gas 3000000 \
  --gas-prices 0.005uc4e \
  --broadcast-mode block

# Wait and verify
sleep 30
c4ed query tx <store_tx_hash>

# Instantiate with production parameters
c4ed tx wasm instantiate $CODE_ID "$(cat instantiate_mainnet.json)" \
  --from deployer \
  --label "detrack-mainnet-v0.1.0" \
  --gas 1800000 \
  --gas-prices 0.005uc4e \
  --broadcast-mode block \
  --no-admin
```

#### 3. Post-Deployment Verification

```bash
# Verify contract deployed correctly
c4ed query wasm contract $CONTRACT_ADDRESS

# Test all query endpoints
./scripts/verify_deployment.sh $CONTRACT_ADDRESS

# Monitor first transactions
c4ed query txs --events "wasm._contract_address='$CONTRACT_ADDRESS'" --limit 10
```

## Post-Deployment Configuration

### 1. Document Deployment Details

Create deployment record `deployments/mainnet.json`:

```json
{
  "deployment_info": {
    "network": "perun-1",
    "code_id": 123,
    "contract_address": "c4e1contract...",
    "deployer": "c4e1deployer...",
    "deployment_time": "2024-11-22T10:00:00Z",
    "contract_version": "0.1.0",
    "git_commit": "abc123def456...",
    "optimization_version": "0.15.0",
    "audit_report": "ipfs://QmAuditReport..."
  },
  "parameters": {
    "min_stake_tier1": "1000000000",
    "min_stake_tier2": "5000000000",
    "min_stake_tier3": "10000000000",
    "deposit_tier1": "100000000",
    "deposit_tier2": "200000000",
    "deposit_tier3": "300000000",
    "use_whitelist": false,
    "deposit_unlock_period_blocks": 100800
  }
}
```

### 2. Configure Treasury (Admin Operation)

```bash
# Set treasury address for receiving slashed funds
c4ed tx wasm execute $CONTRACT_ADDRESS '{
  "admin": {
    "configure_treasury": {
      "treasury_address": "c4e1treasury..."
    }
  }
}' --from deployer --gas auto
```

### 3. Set Up Monitoring

#### Contract Events Monitoring

```bash
# Monitor all contract events
c4ed query txs \
  --events "wasm._contract_address='$CONTRACT_ADDRESS'" \
  --limit 50

# Subscribe to real-time events (WebSocket)
wscat -c wss://rpc.c4e.io/websocket
> {"jsonrpc":"2.0","method":"subscribe","id":1,"params":{"query":"wasm._contract_address='$CONTRACT_ADDRESS'"}}
```

#### Health Check Script

Create `scripts/health_check.sh`:

```bash
#!/bin/bash

CONTRACT_ADDRESS="$1"

if [ -z "$CONTRACT_ADDRESS" ]; then
  echo "Usage: $0 <contract_address>"
  exit 1
fi

echo "🔍 Checking contract health..."

# Test config query
CONFIG=$(c4ed query wasm contract-state smart $CONTRACT_ADDRESS '{"config":{}}' --output json 2>&1)

if [ $? -eq 0 ]; then
  echo "✅ Contract is responding"
  echo "📊 Proof count: $(echo $CONFIG | jq -r '.proof_count')"
  echo "👤 Admin: $(echo $CONFIG | jq -r '.admin')"
  echo "📦 Version: $(echo $CONFIG | jq -r '.version')"
  exit 0
else
  echo "❌ Contract health check failed"
  echo "Error: $CONFIG"
  exit 1
fi
```

Make executable and run:
```bash
chmod +x scripts/health_check.sh
./scripts/health_check.sh $CONTRACT_ADDRESS
```

#### Automated Monitoring with Cron

```bash
# Add to crontab (check every 15 minutes)
crontab -e

# Add line:
*/15 * * * * /path/to/scripts/health_check.sh $CONTRACT_ADDRESS >> /var/log/detrack_health.log 2>&1
```

### 4. Integration Testing

#### End-to-End Test Suite

Create `scripts/e2e_test.sh`:

```bash
#!/bin/bash

set -e

CONTRACT_ADDRESS="$1"
TEST_ACCOUNT="$2"

if [ -z "$CONTRACT_ADDRESS" ] || [ -z "$TEST_ACCOUNT" ]; then
  echo "Usage: $0 <contract_address> <test_account_name>"
  exit 1
fi

TEST_ADDR=$(c4ed keys show $TEST_ACCOUNT -a)

echo "🧪 Running end-to-end tests..."
echo "Contract: $CONTRACT_ADDRESS"
echo "Test Account: $TEST_ADDR"

# Test 1: Query Config
echo ""
echo "Test 1: Query Configuration"
c4ed query wasm contract-state smart $CONTRACT_ADDRESS '{"config":{}}' --output json | jq
echo "✅ Config query successful"

# Test 2: Check if test account is registered
echo ""
echo "Test 2: Check Node Registration Status"
IS_REGISTERED=$(c4ed query wasm contract-state smart $CONTRACT_ADDRESS '{
  "is_whitelisted": {
    "address": "'$TEST_ADDR'"
  }
}' --output json | jq -r '.is_whitelisted')
echo "Registered: $IS_REGISTERED"
echo "✅ Registration check successful"

# Test 3: Query Node Info
echo ""
echo "Test 3: Query Node Information"
c4ed query wasm contract-state smart $CONTRACT_ADDRESS '{
  "node_info": {
    "address": "'$TEST_ADDR'"
  }
}' --output json | jq
echo "✅ Node info query successful"

# Test 4: List proofs
echo ""
echo "Test 4: List Proofs"
c4ed query wasm contract-state smart $CONTRACT_ADDRESS '{
  "proofs": {
    "limit": 5
  }
}' --output json | jq
echo "✅ Proof listing successful"

echo ""
echo "✅ All end-to-end tests passed!"
```

Run tests:
```bash
chmod +x scripts/e2e_test.sh
./scripts/e2e_test.sh $CONTRACT_ADDRESS deployer
```

## Node Operator Setup

### 1. Stake Native Tokens

```bash
# Get validator address
c4ed query staking validators --output json | jq -r '.validators[0].operator_address'

# Delegate tokens (for Tier 1: 1 C4E minimum)
c4ed tx staking delegate c4evaloper1... 1500000000uc4e \
  --from node_operator \
  --gas auto
```

### 2. Register as Node

```bash
# Calculate required deposit based on stake
# For 1.5 C4E staked (Tier 1): need 0.1 C4E deposit

c4ed tx wasm execute $CONTRACT_ADDRESS '{
  "node": {
    "register_node": {}
  }
}' \
  --from node_operator \
  --amount 100000000uc4e \
  --gas auto
```

### 3. Verify Registration

```bash
NODE_ADDR=$(c4ed keys show node_operator -a)

c4ed query wasm contract-state smart $CONTRACT_ADDRESS '{
  "node_info": {
    "address": "'$NODE_ADDR'"
  }
}' | jq
```

## Upgrade Procedures

### Contract Migration (if admin enabled)

```bash
# Store new version
NEW_CODE_ID=$(c4ed tx wasm store artifacts/detrack_node_contract_v2.wasm \
  --from admin \
  --gas auto \
  --output json | jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')

# Prepare migration message
cat > migrate.json << EOF
{
  "migrate": {
    "new_version": "0.2.0"
  }
}
EOF

# Execute migration
c4ed tx wasm migrate $CONTRACT_ADDRESS $NEW_CODE_ID "$(cat migrate.json)" \
  --from admin \
  --gas auto
```

### New Deployment (if no admin)

Since the contract is deployed with `--no-admin`, upgrades require new deployment:

```bash
# 1. Deploy new version
./scripts/deploy.sh mainnet v0.2.0

# 2. Update frontend/backend configurations
# 3. Notify users of new contract address
# 4. Maintain both contracts during transition period
# 5. Eventually deprecate old contract
```

## Security Best Practices

### 1. Key Management

**Hardware Wallet (Recommended for Mainnet)**:
```bash
# Use Ledger device
c4ed keys add deployer --ledger

# Verify address
c4ed keys show deployer -a
```

**Key Backup**:
```bash
# Export key (DO NOT commit to git!)
c4ed keys export deployer > deployer_key_encrypted.txt

# Securely store mnemonic phrase:
# - Write on paper, store in safe
# - Use password manager with encryption
# - Never store in plaintext
# - Never share via insecure channels
```

### 2. Access Control

**Admin Operations**:
- Use multi-signature for critical admin actions
- Document all admin operations
- Rotate keys regularly
- Implement time-locks for critical changes

**Example: Multi-Sig Setup**:
```bash
# Create multi-sig account
c4ed keys add multisig1 --multisig admin1,admin2,admin3 --multisig-threshold 2

# Use multi-sig for admin operations
c4ed tx wasm execute $CONTRACT_ADDRESS '...' --from multisig1
```

### 3. Monitoring & Alerting

**Set Up Alerts**:
```bash
# Monitor for large deposit withdrawals
# Monitor reputation changes
# Alert on failed transactions
# Track gas usage patterns
# Monitor contract balance
```

### 4. Backup and Recovery

**State Backup Script** (`scripts/backup_state.sh`):
```bash
#!/bin/bash

CONTRACT_ADDRESS="$1"
BACKUP_DIR="backups/$(date +%Y%m%d_%H%M%S)"

mkdir -p $BACKUP_DIR

echo "📦 Backing up contract state..."

# Backup config
c4ed query wasm contract-state smart $CONTRACT_ADDRESS '{"config":{}}' \
  > $BACKUP_DIR/config.json

# Backup all proofs (paginated)
START_AFTER=0
LIMIT=30
PAGE=1

while true; do
  PROOFS=$(c4ed query wasm contract-state smart $CONTRACT_ADDRESS '{
    "proofs": {
      "start_after": '$START_AFTER',
      "limit": '$LIMIT'
    }
  }')
  
  echo "$PROOFS" > $BACKUP_DIR/proofs_page_$PAGE.json
  
  # Check if we got less than LIMIT results (last page)
  COUNT=$(echo "$PROOFS" | jq '.proofs | length')
  if [ "$COUNT" -lt "$LIMIT" ]; then
    break
  fi
  
  # Update for next page
  START_AFTER=$(echo "$PROOFS" | jq -r '.proofs[-1].id')
  PAGE=$((PAGE + 1))
done

echo "✅ Backup complete: $BACKUP_DIR"
```

## Troubleshooting

### Common Issues

#### 1. Insufficient Gas

**Error**: `out of gas in location`

**Solution**:
```bash
# Increase gas limit
c4ed tx wasm execute $CONTRACT_ADDRESS '...' \
  --from deployer \
  --gas 3000000  # Increased from default
```

#### 2. Invalid Address Format

**Error**: `invalid address format`

**Solution**:
```bash
# Verify address encoding
c4ed debug addr $(c4ed keys show deployer -a)

# Ensure correct network prefix (c4e1...)
```

#### 3. Code ID Not Found

**Error**: `code id not found`

**Solution**:
```bash
# Verify code was stored successfully
c4ed query wasm list-code

# Check specific code
c4ed query wasm code $CODE_ID
```

#### 4. Insufficient Stake for Registration

**Error**: `InsufficientStake`

**Solution**:
```bash
# Check current stake
c4ed query staking delegations $(c4ed keys show node_operator -a)

# Delegate more tokens
c4ed tx staking delegate <validator> <amount>uc4e --from node_operator
```

#### 5. Deposit Doesn't Match Tier Requirement

**Error**: `DepositDoesNotMatchTierRequirement`

**Solution**:
```bash
# Query config to see tier requirements
c4ed query wasm contract-state smart $CONTRACT_ADDRESS '{"config":{}}'

# Send correct deposit amount for your tier
```

### Debugging Commands

#### Contract State

```bash
# Check all contract state
c4ed query wasm contract-state all $CONTRACT_ADDRESS

# Check raw state (for specific keys)
c4ed query wasm contract-state raw $CONTRACT_ADDRESS <hex_key>
```

#### Transaction Analysis

```bash
# Get transaction details
c4ed query tx <tx_hash> --output json | jq

# Check events
c4ed query tx <tx_hash> --output json | jq '.logs[].events[]'

# View gas used
c4ed query tx <tx_hash> --output json | jq '.gas_used'
```

#### Gas Estimation

```bash
# Simulate transaction
c4ed tx wasm execute $CONTRACT_ADDRESS '...' \
  --from deployer \
  --dry-run

# Get gas usage estimate
c4ed tx wasm execute $CONTRACT_ADDRESS '...' \
  --from deployer \
  --gas-adjustment 1.0 \
  --simulate
```

## Appendix

### A. Network Endpoints Reference

| Network | Chain ID | RPC | REST | Explorer | Faucet |
|---------|----------|-----|------|----------|--------|
| Mainnet | perun-1 | https://rpc.c4e.io:443 | https://lcd.c4e.io:443 | https://explorer.c4e.io | N/A |
| Testnet | babajaga-1 | https://rpc.babajaga.c4e.io:443 | https://lcd.babajaga.c4e.io:443 | https://testnet.explorer.c4e.io | https://faucet.babajaga.c4e.io |
| Local | c4e-dev | http://localhost:26657 | http://localhost:1317 | N/A | Local |

### B. Gas Estimation Guide

| Operation | Estimated Gas | Notes |
|-----------|---------------|-------|
| Store Code | 2,500,000 - 3,500,000 | Depends on contract size (~600KB) |
| Instantiate | 500,000 - 800,000 | Simple instantiation |
| Register Node | 250,000 - 400,000 | Includes staking query |
| Store Proof | 200,000 - 350,000 | Depends on metadata size |
| Add Deposit | 120,000 - 180,000 | Simple state update |
| Unlock Deposit | 150,000 - 220,000 | State reorganization |
| Claim Deposit | 180,000 - 250,000 | Bank transfer included |
| Query Operations | 50,000 - 150,000 | Read-only, depends on complexity |

### C. Deployment Script Template

Create `scripts/deploy.sh`:

```bash
#!/bin/bash

set -e

NETWORK=${1:-testnet}
VERSION=${2:-0.1.0}

if [ "$NETWORK" = "mainnet" ]; then
  CHAIN_ID="perun-1"
  NODE="https://rpc.c4e.io:443"
  GAS_PRICES="0.005uc4e"
elif [ "$NETWORK" = "testnet" ]; then
  CHAIN_ID="babajaga-1"
  NODE="https://rpc.babajaga.c4e.io:443"
  GAS_PRICES="0.0025uc4e"
else
  echo "Invalid network: $NETWORK"
  exit 1
fi

DEPLOYER="deployer"
DEPLOYER_ADDR=$(c4ed keys show $DEPLOYER -a)

echo "🚀 Deploying DeTrack Node Contract"
echo "Network: $NETWORK ($CHAIN_ID)"
echo "Version: $VERSION"
echo "Deployer: $DEPLOYER_ADDR"
echo ""

# Build optimized WASM
echo "📦 Building optimized WASM..."
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.15.0

# Store code
echo ""
echo "📤 Storing contract code..."
STORE_TX=$(c4ed tx wasm store artifacts/detrack_node_contract.wasm \
  --from $DEPLOYER \
  --node $NODE \
  --chain-id $CHAIN_ID \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $GAS_PRICES \
  --broadcast-mode block \
  --output json \
  --yes)

CODE_ID=$(echo $STORE_TX | jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
echo "✅ Code stored with ID: $CODE_ID"

# Instantiate
echo ""
echo "🎬 Instantiating contract..."
INIT_TX=$(c4ed tx wasm instantiate $CODE_ID "$(cat config/instantiate_${NETWORK}.json)" \
  --from $DEPLOYER \
  --node $NODE \
  --chain-id $CHAIN_ID \
  --label "detrack-node-$NETWORK-$VERSION" \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $GAS_PRICES \
  --no-admin \
  --broadcast-mode block \
  --output json \
  --yes)

CONTRACT_ADDRESS=$(echo $INIT_TX | jq -r '.logs[0].events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value')
echo "✅ Contract instantiated at: $CONTRACT_ADDRESS"

# Save deployment info
echo ""
echo "💾 Saving deployment info..."
cat > deployments/${NETWORK}_${VERSION}.json << EOF
{
  "network": "$NETWORK",
  "chain_id": "$CHAIN_ID",
  "code_id": "$CODE_ID",
  "contract_address": "$CONTRACT_ADDRESS",
  "deployer": "$DEPLOYER_ADDR",
  "deployment_time": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "version": "$VERSION"
}
EOF

echo "✅ Deployment complete!"
echo ""
echo "Contract Address: $CONTRACT_ADDRESS"
echo "Code ID: $CODE_ID"
echo ""
echo "Next steps:"
echo "1. Verify deployment: ./scripts/e2e_test.sh $CONTRACT_ADDRESS"
echo "2. Configure treasury: c4ed tx wasm execute $CONTRACT_ADDRESS '{\"admin\":{\"configure_treasury\":{\"treasury_address\":\"...\"}}}' --from $DEPLOYER"
echo "3. Update documentation with contract address"
```

Make executable:
```bash
chmod +x scripts/deploy.sh
```

Usage:
```bash
# Deploy to testnet
./scripts/deploy.sh testnet 0.1.0

# Deploy to mainnet
./scripts/deploy.sh mainnet 0.1.0
```

### D. Environment File Template

Create `.env.example`:
```bash
# Network Configuration
NETWORK=testnet
CHAIN_ID=babajaga-1
NODE=https://rpc.babajaga.c4e.io:443

# Contract Addresses
CONTRACT_ADDRESS=c4e1...

# Deployer Configuration
DEPLOYER=deployer
DEPLOYER_ADDRESS=c4e1...

# Gas Configuration
GAS_PRICES=0.0025uc4e
GAS_ADJUSTMENT=1.3

# Monitoring
HEALTH_CHECK_INTERVAL=900  # 15 minutes
```

## Support

- **Issues**: [GitHub Issues](https://github.com/chain4energy/detrack-node-contract/issues)
- **Discord**: [Chain4Energy Discord](https://discord.gg/c4e)
- **Documentation**: [docs/](https://github.com/chain4energy/detrack-node-contract/tree/main/docs)
- **Website**: [https://c4e.io](https://c4e.io)

## Version History

- **v0.1.0** (Initial Release)
  - Node registration with tiered staking
  - Proof storage and verification
  - Deposit locking with unbonding period
  - Admin operations
  - User registry

For detailed changes, see [CHANGELOG.md](../CHANGELOG.md).
