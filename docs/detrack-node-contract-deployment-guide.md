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
- **RPC Endpoint**: `https://rpc-testnet.c4e.io:443`
- **REST Endpoint**: `https://lcd-testnet.c4e.io:443`
- **Gas Prices**: `0.0025uc4e`
- **Faucet**: `https://faucet-testnet.c4e.io`
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
c4ed config node https://rpc-testnet.c4e.io:443
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
  "version": "v0.3.3",
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
  --label "detrack-node-v0.3.3" \
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
  "version": "v0.3.3",
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

