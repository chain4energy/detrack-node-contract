# DeTrack Smart Contract (CosmWASM)

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

This is a CosmWASM smart contract for the DeTrack project on the c4e chain. It provides functionality for storing and verifying cryptographic proofs of energy data.

## Overview

The DeTrack smart contract implements a decentralized node management and proof verification system for energy data on the Chain4Energy blockchain.

**Current Version:** Phase 1a  
**Target Version:** Phase 1b (DID-First Architecture - Variant C)  
**Status:** Roadmap planning phase

### Phase 1a (Current Implementation)

The contract currently allows:
1. Storing cryptographic proofs of energy data on the blockchain
2. Verifying the existence of proofs by data hash
3. Managing nodes with tiered staking requirements
4. Querying proofs by various parameters
5. Admin-controlled whitelist system

### Phase 1b (Target - Variant C)

The planned upgrade will introduce:
- ✅ **DID-Based Trust Chain:** W3C-compliant DID verification (Gateway → Worker → Blockchain)
- ✅ **Multi-Batch Aggregation:** 8+ batches per proof (99.94% cost reduction)
- ✅ **Secondary Indexes:** Efficient queries by worker_did and gateway_did
- ✅ **Gas Optimization:** Remove redundant fields (~210 gas savings per proof)
- ✅ **Future-Proof:** Ready for Device DID (Phase 3) and NFT ownership (Phase 4)

📜 **See the complete roadmap:** [docs/detrack-node-contract-roadmap.md](./docs/detrack-node-contract-roadmap.md)

## Contract Structure

- `src/contract.rs` - Main contract entry points (instantiate, execute, query)
- `src/error.rs` - Error handling
- `src/execute.rs` - Execute message handlers
- `src/msg.rs` - Message definitions
- `src/query.rs` - Query handlers
- `src/state.rs` - State management

## Message Types

**Note:** The message types below represent the **Phase 1a (current)** implementation. For the **Phase 1b (target)** architecture with DID integration and multi-batch aggregation, see the [Migration Roadmap](./docs/detrack-node-contract-roadmap.md).

### InstantiateMsg
```rust
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub version: String,
}
```

### ExecuteMsg
```rust
pub enum ExecuteMsg {
    // Admin operations
    Admin(AdminExecuteMsg),
    // Node operations
    Node(NodeExecuteMsg),
}

// Admin operations
pub enum AdminExecuteMsg {
    // Update the admin address
    UpdateAdmin { new_admin: String },
    // Whitelist a node address
    WhitelistNode { node_address: String },
    // Remove a node from the whitelist
    RemoveNode { node_address: String },
    // Update node reputation
    UpdateNodeReputation { node_address: String, reputation: i32 },
    // Update the minimum reputation threshold
    UpdateMinReputationThreshold { threshold: i32 },
}

// Node operations
pub enum NodeExecuteMsg {
    // Store a new proof on the blockchain
    StoreProof {
        data_hash: String,
        original_data_reference: Option<String>,
        data_owner: Option<String>,
        metadata_json: Option<String>,
    },
    // Register a new user
    RegisterUser {},
    // Verify a proof
    VerifyProof { data_hash: String },
}
```

### QueryMsg
```rust
pub enum QueryMsg {
    Config {},
    Proof { id: u64 },
    ProofByHash { data_hash: String },
    Proofs { start_after: Option<u64>, limit: Option<u32> },
    User { address: String },
    UserProofs { address: String, start_after: Option<u64>, limit: Option<u32> },
    // Check if an address is whitelisted as a node
    IsWhitelisted { address: String },
    // Get a node's reputation score
    NodeReputation { address: String },
}
```

## Building and Deploying

```bash
# Build the contract
cargo build
```

```bash
# Run tests
cargo test
```

```bash
# Create optimized build using Docker
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.14.0

#HINT
# comment version = 4 in Cargo.lock if you have error during optimization

# Alternative: if you have issues with the Docker command above, you can use the manual optimization process:
RUSTFLAGS='-C link-arg=-s' cargo wasm
```

## Deployment Guide

### 1. Set Up Environment Variables
```bash
# Set environment variables for convenience
CHAIN_ID="c4e-chain-compose"       # e.g., "c4e-testnet"
NODE=http://localhost:31657
HOME_DIR="/home/greg/projects/c4e/test-c4e-chain-docker/client-home"
KEY_NAME="alice"               # Your key name
WASM_FILE="artifacts/detrack_contract.wasm"
APP_ADMIN="c4e1yyjfd5cj5nd0jrlvrhc5p3mnkcn8v9q8fdd9gs" # Your app admin address
APP_NAME="DeTrack" # Your app name
APP_VERSION="0.1.0" # Your app version
APP_DESCRIPTION="DeTrack smart contract for energy data" # Your app description
ADMIN_ADDRESS="c4e1yyjfd5cj5nd0jrlvrhc5p3mnkcn8v9q8fdd9gs" # Your admin address
```

### 2. Store Contract on Chain
```bash
# Upload the WASM file to the blockchain
c4ed --home $HOME_DIR tx wasm store $WASM_FILE \
  --from $KEY_NAME \
  --chain-id $CHAIN_ID \
  --gas auto \
  --gas-adjustment 1.3 \
  --broadcast-mode sync \
  -y
```

### 3. Query the Transaction to Get the Code ID
```bash
# Query the transaction using the txhash from the previous command output
c4ed --home $HOME_DIR query tx YOUR_TX_HASH
```

### 4. Instantiate the Contract
```bash
# Deploy the contract with parameters
c4ed --home $HOME_DIR tx wasm instantiate 3 \
  '{"admin": "'$APP_ADMIN'", "version": "'$APP_VERSION'"}' \
  --from $KEY_NAME \
  --label "'$APP_NAME'" \
  --admin $ADMIN_ADDRESS \
  --chain-id $CHAIN_ID \
  --broadcast-mode sync \
  -y
```
Note: Replace `2` with your actual code ID from the previous step, and update the marketplace owner address.

### 5. Query the Contract Address
```bash
# Find your contract's address
c4ed --home $HOME_DIR query wasm list-contract-by-code 3
```
Note: Replace `2` with your actual code ID.


## Integration with DeTrack Node

This contract is designed to be used with the DeTrack Node server, which verifies energy data and submits proofs to the blockchain.

Example of submitting a proof from TypeScript/JavaScript:
```typescript
const msg = {
  node: {
    store_proof: {
      data_hash: proofHash,
      original_data_reference: originalDataReference,
      data_owner: dataOwner,
      metadata_json: JSON.stringify(metadataForSC)
    }
  }
};

// Execute the contract using CosmJS
const result = await signingClient.execute(
  senderAddress,
  contractAddress,
  msg,
  "auto"
);
```

## Testing the Contract

After deploying the contract, you can test its functionality using the provided test scripts:

### Using thdo verify prooe Bash Script

```bash
# Make the script executable
chmod +x test-contract.sh

# Run the test script
./test-contract.sh
```

This script tests all the main contract functions:
1. Store a proof on the blockchain
2. Query the proof by its hash
3. Verify the proof exists
4. Query the contract configuration
5. Register a new user
6. Query the user's information

### Using the TypeScript Test

```bash
# Install dependencies
npm install

# Run the TypeScript test
npm run test:contract
```

The TypeScript test provides similar functionality to the Bash script but uses the CosmJS client directly, which is the same approach used by the DeTrack Node server.

### Examples of Admin Operations

```typescript
// Example: Whitelist a node
const whitelistMsg = {
  admin: {
    whitelist_node: {
      node_address: "c4e1..."
    }
  }
};

// Example: Update node reputation
const updateReputationMsg = {
  admin: {
    update_node_reputation: {
      node_address: "c4e1...",
      reputation: 75
    }
  }
};

// Example: Update minimum reputation threshold
const updateThresholdMsg = {
  admin: {
    update_min_reputation_threshold: {
      threshold: 50
    }
  }
};

// Execute admin operations using CosmJS
const result = await signingClient.execute(
  adminAddress,  // Must be the admin address
  contractAddress,
  adminMsg,
  "auto"
);
```

### Node Access Control

The contract implements role-based access control:
- Only whitelisted nodes can store proofs and perform node operations
- Admin operations require the admin address
- Nodes must meet tiered staking requirements

## Documentation

### Current Implementation (Phase 1a)
- [API Specification](./docs/api-specification.md) - Complete API reference
- [Contract Design](./docs/contract-design.md) - Design overview
- [Deployment Guide](./docs/deployment-guide.md) - Deployment instructions

### Roadmap to Phase 1b
- [Migration Roadmap](./docs/detrack-node-contract-roadmap.md) - **Complete roadmap to Variant C (DID-First Architecture)**

### Related Documentation
- [DeTrack Worker Node](../detrack-worker-node/README.md) - Worker Node implementation
- [STORE-PROOF-ANALYSIS](../detrack-worker-node/docs/STORE-PROOF-ANALYSIS.md) - Variant C architecture analysis
- [ADR-003: Remove Aggregation Fields](../detrack-worker-node/docs/ADR/ADR-003-remove-aggregation-fields.md)
- [ADR-004: DID Integration](../detrack-worker-node/docs/ADR/ADR-004-did-integration.md)

## License

Apache-2.0
- Nodes must maintain a reputation score above the minimum threshold
- Only the admin can whitelist/remove nodes and update reputation scores

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute to this project.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Support

For support and questions:
- Open an issue on [GitHub Issues](https://github.com/chain4energy/detrack-node-contract/issues)
- Visit the [Chain4Energy organization](https://github.com/chain4energy)

## Related Projects

- [DeTrack Worker Node](https://github.com/chain4energy/detrack-worker-node) - Node implementation for the DeTrack network
- [DID Contract](https://github.com/chain4energy/did-contract) - Decentralized Identity contract for c4e chain
