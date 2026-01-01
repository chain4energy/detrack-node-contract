# DeTrack Node Contract API Specification

## Overview

This document provides a comprehensive API reference for the DeTrack Node Contract. The contract implements a decentralized node management and proof verification system for energy data on the Chain4Energy blockchain.

## Contract Interface

### Instantiation

The contract requires configuration parameters for the tiered node system and deposit management.

#### Instantiate Message

```json
{
  "admin": "c4e1admin...",
  "version": "0.1.0",
  "min_stake_tier1": "1000000000",
  "min_stake_tier2": "5000000000",
  "min_stake_tier3": "10000000000",
  "deposit_tier1": "100000000",
  "deposit_tier2": "200000000",
  "deposit_tier3": "300000000",
  "use_whitelist": false,
  "deposit_unlock_period_blocks": 100800
}
```

#### Parameters
- `admin` (optional, string): Admin address. If null, instantiator becomes admin
- `version` (string): Contract version identifier
- `min_stake_tier1` (Uint128): Minimum native stake for Tier 1 nodes
- `min_stake_tier2` (Uint128): Minimum native stake for Tier 2 nodes
- `min_stake_tier3` (Uint128): Minimum native stake for Tier 3 nodes
- `deposit_tier1` (Uint128): Required contract deposit for Tier 1 (in uc4e)
- `deposit_tier2` (Uint128): Required contract deposit for Tier 2
- `deposit_tier3` (Uint128): Required contract deposit for Tier 3
- `use_whitelist` (bool): Whether nodes must be admin-whitelisted before registration
- `deposit_unlock_period_blocks` (u64): Unbonding period in blocks (e.g., 100800 ≈ 7 days)

#### Example

```bash
c4ed tx wasm instantiate <code_id> '{
  "admin": null,
  "version": "0.1.0",
  "min_stake_tier1": "1000000000",
  "min_stake_tier2": "5000000000",
  "min_stake_tier3": "10000000000",
  "deposit_tier1": "100000000",
  "deposit_tier2": "200000000",
  "deposit_tier3": "300000000",
  "use_whitelist": false,
  "deposit_unlock_period_blocks": 100800
}' \
  --from deployer \
  --label "detrack-node-v0.1.0" \
  --gas auto \
  --gas-adjustment 1.5 \
  --no-admin
```

## Execute Messages

Execute messages are wrapped in either `Admin` or `Node` enums for clear permission separation.

### Admin Execute Messages

Admin operations accessible only by the contract admin.

#### 1. Update Admin

Changes the contract's admin address.

```json
{
  "admin": {
    "update_admin": {
      "new_admin": "c4e1newadmin..."
    }
  }
}
```

**Authorization**: Current admin only

**Example**:
```bash
c4ed tx wasm execute <contract_addr> '{
  "admin": {
    "update_admin": {
      "new_admin": "c4e1newadmin..."
    }
  }
}' --from admin --gas auto
```

**Response Events**:
```json
{
  "type": "wasm",
  "attributes": [
    {"key": "action", "value": "update_admin"},
    {"key": "new_admin", "value": "c4e1newadmin..."}
  ]
}
```

#### 2. Whitelist Node

Adds a node address to the whitelist with tier 0 (non-operational) status. Node must still register with deposit to become operational.

```json
{
  "admin": {
    "whitelist_node": {
      "node_address": "c4e1node..."
    }
  }
}
```

**Authorization**: Admin only

**Parameters**:
- `node_address` (string): Address to whitelist

**Validation**:
- Node address must be valid C4E address
- Node must not already be whitelisted

**Example**:
```bash
c4ed tx wasm execute <contract_addr> '{
  "admin": {
    "whitelist_node": {
      "node_address": "c4e1node..."
    }
  }
}' --from admin --gas auto
```

**Response Events**:
```json
{
  "type": "wasm",
  "attributes": [
    {"key": "action", "value": "whitelist_node"},
    {"key": "node_address", "value": "c4e1node..."}
  ]
}
```

**Errors**:
- `AdminOnlyOperation`: Caller is not admin
- `NodeAlreadyWhitelisted`: Node already in whitelist

#### 3. Remove Node

Removes a node from the registry. Node should unlock deposit first.

```json
{
  "admin": {
    "remove_node": {
      "node_address": "c4e1node..."
    }
  }
}
```

**Authorization**: Admin only

**Note**: This removes the node entry but does not automatically return their deposit. The node must call `UnlockDeposit` and `ClaimUnlockedDeposit` separately.

**Example**:
```bash
c4ed tx wasm execute <contract_addr> '{
  "admin": {
    "remove_node": {
      "node_address": "c4e1node..."
    }
  }
}' --from admin --gas auto
```

**Errors**:
- `AdminOnlyOperation`: Caller is not admin
- `NodeNotWhitelisted`: Node not found in registry

#### 4. Update Node Reputation

Updates the reputation score for a registered node.

```json
{
  "admin": {
    "update_node_reputation": {
      "node_address": "c4e1node...",
      "reputation": 100
    }
  }
}
```

**Authorization**: Admin only

**Parameters**:
- `node_address` (string): Target node address
- `reputation` (i32): New reputation score (can be negative)

**Example**:
```bash
c4ed tx wasm execute <contract_addr> '{
  "admin": {
    "update_node_reputation": {
      "node_address": "c4e1node...",
      "reputation": 100
    }
  }
}' --from admin --gas auto
```

**Response Events**:
```json
{
  "type": "wasm",
  "attributes": [
    {"key": "action", "value": "update_node_reputation"},
    {"key": "node_address", "value": "c4e1node..."},
    {"key": "reputation", "value": "100"}
  ]
}
```

#### 5. Update Minimum Reputation Threshold

Updates the global minimum reputation required for nodes to store proofs.

```json
{
  "admin": {
    "update_min_reputation_threshold": {
      "threshold": 0
    }
  }
}
```

**Authorization**: Admin only

**Parameters**:
- `threshold` (i32): New minimum reputation threshold

**Example**:
```bash
c4ed tx wasm execute <contract_addr> '{
  "admin": {
    "update_min_reputation_threshold": {
      "threshold": 10
    }
  }
}' --from admin --gas auto
```

**Response Events**:
```json
{
  "type": "wasm",
  "attributes": [
    {"key": "action", "value": "update_min_reputation_threshold"},
    {"key": "threshold", "value": "10"}
  ]
}
```

#### 6. Configure Treasury

Sets or updates the treasury address for receiving slashed funds.

```json
{
  "admin": {
    "configure_treasury": {
      "treasury_address": "c4e1treasury..."
    }
  }
}
```

**Authorization**: Admin only

**Parameters**:
- `treasury_address` (string): Treasury wallet address

**Example**:
```bash
c4ed tx wasm execute <contract_addr> '{
  "admin": {
    "configure_treasury": {
      "treasury_address": "c4e1treasury..."
    }
  }
}' --from admin --gas auto
```

**Response Events**:
```json
{
  "type": "wasm",
  "attributes": [
    {"key": "method", "value": "configure_treasury"},
    {"key": "treasury", "value": "c4e1treasury..."}
  ]
}
```

### Node Execute Messages

Operations available to registered nodes and users.

#### 1. Register Node

Registers a new node by verifying native stake and locking deposit.

```json
{
  "node": {
    "register_node": {}
  }
}
```

**Authorization**: Anyone (subject to whitelist if enabled)

**Required Funds**: Must send deposit matching tier (determined by stake)

**Process**:
1. Contract queries staking module for sender's total native stake
2. Determines tier based on stake amount:
   - Tier 3: stake >= `min_stake_tier3`
   - Tier 2: stake >= `min_stake_tier2`
   - Tier 1: stake >= `min_stake_tier1`
3. Verifies sent deposit matches tier requirement
4. Creates/updates node entry with operational tier

**Example** (Tier 1 registration):
```bash
# Sender has 1.5 C4E staked (1500000000 uc4e)
# Qualifies for Tier 1, must send 100 C4E deposit (100000000 uc4e)
c4ed tx wasm execute <contract_addr> '{
  "node": {
    "register_node": {}
  }
}' \
  --from node_operator \
  --amount 100000000uc4e \
  --gas auto
```

**Response Events**:
```json
{
  "type": "wasm",
  "attributes": [
    {"key": "action", "value": "register_node"},
    {"key": "node_address", "value": "c4e1node..."},
    {"key": "native_stake_verified", "value": "1500000000"},
    {"key": "tier_assigned", "value": "1"},
    {"key": "deposit_locked", "value": "100000000"}
  ]
}
```

**Errors**:
- `CustomError("Node already registered")`: Node already has operational tier
- `InsufficientStake`: Native stake below minimum for Tier 1
- `DepositDoesNotMatchTierRequirement`: Sent deposit doesn't match tier
- `NodeNotWhitelisted`: Whitelist mode enabled but node not whitelisted

#### 2. Store Proof

Stores a cryptographic proof of energy data on-chain.

```json
{
  "node": {
    "store_proof": {
      "data_hash": "0xabc123...",
      "original_data_reference": "ipfs://Qm...",
      "data_owner": "c4e1owner...",
      "metadata_json": "{\"device_id\":\"meter-001\"}",
      "tw_start": "2024-11-01T00:00:00Z",
      "tw_end": "2024-11-01T01:00:00Z",
      "value_in": "1500000",
      "value_out": "1200000",
      "unit": "Wh"
    }
  }
}
```

**Authorization**: Registered nodes with:
- Reputation >= `min_reputation_threshold`
- Operational tier (1-3)
- Deposit >= tier requirement

**Parameters**:
- `data_hash` (string): SHA-256 or similar hash of energy data (required, non-empty)
- `original_data_reference` (optional, string): Reference to full data (e.g., IPFS CID)
- `data_owner` (optional, string): Address of data owner (defaults to sender)
- `metadata_json` (optional, string): Additional JSON metadata
- `tw_start` (Timestamp): Time window start for energy measurement
- `tw_end` (Timestamp): Time window end for energy measurement
- `value_in` (optional, Uint128): Energy input value in specified units
- `value_out` (optional, Uint128): Energy output value in specified units
- `unit` (string): Unit of measurement (e.g., "Wh", "kWh")

**Validation**:
- `data_hash` must be non-empty
- No duplicate proofs (hash must be unique)
- `data_owner` must be valid address if provided
- Node must meet reputation and deposit requirements

**Example**:
```bash
c4ed tx wasm execute <contract_addr> '{
  "node": {
    "store_proof": {
      "data_hash": "0xabc123def456...",
      "original_data_reference": "ipfs://QmXyz789...",
      "data_owner": "c4e1homeowner...",
      "metadata_json": "{\"device_id\":\"smart-meter-001\",\"location\":\"Building-A\"}",
      "tw_start": "1698796800000000000",
      "tw_end": "1698800400000000000",
      "value_in": "1500000",
      "value_out": "1200000",
      "unit": "Wh"
    }
  }
}' --from node_operator --gas auto
```

**Response Events**:
```json
{
  "type": "wasm",
  "attributes": [
    {"key": "action", "value": "store_proof"},
    {"key": "proof_id", "value": "42"},
    {"key": "data_hash", "value": "0xabc123def456..."},
    {"key": "stored_by", "value": "c4e1node..."},
    {"key": "data_owner", "value": "c4e1homeowner..."}
  ]
}
```

**Errors**:
- `NodeNotWhitelisted`: Node not registered
- `InsufficientNodeReputation`: Reputation below threshold
- `NodeTierNotOperational`: Node tier is 0 (not operational)
- `NodeHasInsufficientDeposit`: Deposit below tier requirement
- `InvalidInput`: Data hash is empty
- `ProofAlreadyExists`: Proof with same hash already exists
- `InvalidDataOwner`: Invalid data owner address

#### 3. Verify Proof

Verifies existence of a proof by its hash.

```json
{
  "node": {
    "verify_proof": {
      "data_hash": "0xabc123..."
    }
  }
}
```

**Authorization**: Registered nodes with sufficient reputation

**Parameters**:
- `data_hash` (string): Hash of proof to verify

**Example**:
```bash
c4ed tx wasm execute <contract_addr> '{
  "node": {
    "verify_proof": {
      "data_hash": "0xabc123def456..."
    }
  }
}' --from node_operator --gas auto
```

**Response Events**:
```json
{
  "type": "wasm",
  "attributes": [
    {"key": "action", "value": "verify_proof"},
    {"key": "verified", "value": "true"},
    {"key": "data_hash", "value": "0xabc123def456..."},
    {"key": "proof_id", "value": "42"}
  ]
}
```

**Errors**:
- `NodeNotWhitelisted`: Node not registered
- `InsufficientNodeReputation`: Reputation below threshold
- `ProofNotFound`: No proof with given hash exists

#### 4. Add Deposit

Adds additional funds to a node's existing deposit.

```json
{
  "node": {
    "add_deposit": {}
  }
}
```

**Authorization**: Registered nodes only

**Required Funds**: Must send uc4e tokens

**Validation**:
- Node must be registered
- Deposit must not be in unlocking state
- Must send positive amount of uc4e

**Example**:
```bash
c4ed tx wasm execute <contract_addr> '{
  "node": {
    "add_deposit": {}
  }
}' \
  --from node_operator \
  --amount 50000000uc4e \
  --gas auto
```

**Response Events**:
```json
{
  "type": "wasm",
  "attributes": [
    {"key": "action", "value": "add_deposit"},
    {"key": "node_address", "value": "c4e1node..."},
    {"key": "added_amount", "value": "50000000"},
    {"key": "new_total_deposit", "value": "150000000"}
  ]
}
```

**Errors**:
- `NodeNotRegistered`: Sender is not a registered node
- `DepositAlreadyUnlocking`: Deposit currently in unbonding period
- `CustomError`: No deposit sent or invalid denomination

#### 5. Unlock Deposit

Initiates the unbonding period for a node's deposit.

```json
{
  "node": {
    "unlock_deposit": {}
  }
}
```

**Authorization**: Registered nodes only

**Process**:
1. Verify node has active deposit
2. Check no existing unlocking deposit
3. Move deposit to `UNLOCKING_DEPOSITS` map
4. Set release block: `current_block + deposit_unlock_period_blocks`
5. Set node's active deposit to zero

**State Change**:
- Node cannot store new proofs (deposit = 0)
- Funds locked in contract for unbonding period
- After release block, funds become claimable

**Example**:
```bash
c4ed tx wasm execute <contract_addr> '{
  "node": {
    "unlock_deposit": {}
  }
}' --from node_operator --gas auto
```

**Response Events**:
```json
{
  "type": "detrack_unlock_deposit",
  "attributes": [
    {"key": "node_address", "value": "c4e1node..."},
    {"key": "unlocking_amount", "value": "100000000"},
    {"key": "release_at_block", "value": "1234567"}
  ]
}
```

**Errors**:
- `NodeNotRegistered`: Sender is not a registered node
- `DepositAlreadyUnlocking`: Already have unlocking deposit
- `NoDepositToUnlock`: Node's active deposit is zero

#### 6. Claim Unlocked Deposit

Claims deposit after unbonding period has passed.

```json
{
  "node": {
    "claim_unlocked_deposit": {}
  }
}
```

**Authorization**: Node who initiated unlock

**Validation**:
- Unlocking deposit must exist
- Current block >= release_at_block

**Process**:
1. Load unlocking deposit entry
2. Verify unbonding period has passed
3. Remove from `UNLOCKING_DEPOSITS`
4. Transfer funds back to node via `BankMsg::Send`

**Example**:
```bash
c4ed tx wasm execute <contract_addr> '{
  "node": {
    "claim_unlocked_deposit": {}
  }
}' --from node_operator --gas auto
```

**Response Events**:
```json
{
  "type": "detrack_claim_unlocked_deposit",
  "attributes": [
    {"key": "node_address", "value": "c4e1node..."},
    {"key": "claimed_amount", "value": "100000000"}
  ]
}
```

**Errors**:
- `NoUnlockedDepositToClaim`: No unlocking deposit entry exists
- `DepositNotYetUnlocked`: Current block < release_at_block

## Query Messages

### 1. Get Config

Retrieves the contract's current configuration.

```json
{
  "config": {}
}
```

**Response**:
```json
{
  "admin": "c4e1admin...",
  "version": "0.1.0",
  "proof_count": 1234,
  "min_reputation_threshold": 0,
  "treasury": "c4e1treasury...",
  "min_stake_tier1": "1000000000",
  "min_stake_tier2": "5000000000",
  "min_stake_tier3": "10000000000",
  "deposit_tier1": "100000000",
  "deposit_tier2": "200000000",
  "deposit_tier3": "300000000",
  "use_whitelist": false,
  "deposit_unlock_period_blocks": 100800
}
```

**Example**:
```bash
c4ed query wasm contract-state smart <contract_addr> '{"config":{}}'
```

### 2. Get Proof by ID

Retrieves a specific proof by its sequential ID.

```json
{
  "proof": {
    "id": 42
  }
}
```

**Response**:
```json
{
  "id": 42,
  "data_hash": "0xabc123def456...",
  "original_data_reference": "ipfs://QmXyz789...",
  "data_owner": "c4e1homeowner...",
  "metadata_json": "{\"device_id\":\"smart-meter-001\"}",
  "stored_at": "1698800400000000000",
  "stored_by": "c4e1node...",
  "tw_start": "1698796800000000000",
  "tw_end": "1698800400000000000",
  "value_in": "1500000",
  "value_out": "1200000",
  "unit": "Wh"
}
```

**Example**:
```bash
c4ed query wasm contract-state smart <contract_addr> '{
  "proof": {"id": 42}
}'
```

**Errors**:
- `NotFound`: Proof ID does not exist

### 3. Get Proof by Hash

Retrieves a specific proof by its data hash.

```json
{
  "proof_by_hash": {
    "data_hash": "0xabc123def456..."
  }
}
```

**Response**: Same as Get Proof by ID

**Example**:
```bash
c4ed query wasm contract-state smart <contract_addr> '{
  "proof_by_hash": {
    "data_hash": "0xabc123def456..."
  }
}'
```

**Errors**:
- `NotFound`: No proof with given hash

### 4. List All Proofs

Lists proofs with pagination support.

```json
{
  "proofs": {
    "start_after": 40,
    "limit": 10
  }
}
```

**Parameters**:
- `start_after` (optional, u64): Proof ID to start after (for pagination)
- `limit` (optional, u32): Maximum results (default: 10, max: 30)

**Response**:
```json
{
  "proofs": [
    {
      "id": 41,
      "data_hash": "0xdef...",
      "stored_by": "c4e1node1...",
      ...
    },
    {
      "id": 42,
      "data_hash": "0xabc...",
      "stored_by": "c4e1node2...",
      ...
    }
  ]
}
```

**Example**:
```bash
c4ed query wasm contract-state smart <contract_addr> '{
  "proofs": {
    "start_after": 40,
    "limit": 10
  }
}'
```

### 5. Get User Profile

Retrieves user information including their proof list.

```json
{
  "user": {
    "address": "c4e1homeowner..."
  }
}
```

**Response**:
```json
{
  "address": "c4e1homeowner...",
  "proofs": [42, 43, 44, 45],
  "registered_at": "1698800400000000000"
}
```

**Example**:
```bash
c4ed query wasm contract-state smart <contract_addr> '{
  "user": {
    "address": "c4e1homeowner..."
  }
}'
```

**Errors**:
- `NotFound`: User not found (never had proofs stored)

### 6. Get User Proofs

Lists all proofs owned by a user with pagination.

```json
{
  "user_proofs": {
    "address": "c4e1homeowner...",
    "start_after": 42,
    "limit": 10
  }
}
```

**Parameters**:
- `address` (string): User address
- `start_after` (optional, u64): Proof ID to start after
- `limit` (optional, u32): Maximum results (default: 10, max: 30)

**Response**: Same format as List All Proofs

**Example**:
```bash
c4ed query wasm contract-state smart <contract_addr> '{
  "user_proofs": {
    "address": "c4e1homeowner...",
    "limit": 10
  }
}'
```

### 7. Check if Node is Whitelisted

Checks if an address is registered in the node registry.

```json
{
  "is_whitelisted": {
    "address": "c4e1node..."
  }
}
```

**Response**:
```json
{
  "is_whitelisted": true
}
```

**Example**:
```bash
c4ed query wasm contract-state smart <contract_addr> '{
  "is_whitelisted": {
    "address": "c4e1node..."
  }
}'
```

### 8. Get Node Reputation

Retrieves a node's reputation score.

```json
{
  "node_reputation": {
    "address": "c4e1node..."
  }
}
```

**Response**:
```json
{
  "address": "c4e1node...",
  "reputation": 100
}
```

**Note**: Returns reputation of 0 for non-registered nodes.

**Example**:
```bash
c4ed query wasm contract-state smart <contract_addr> '{
  "node_reputation": {
    "address": "c4e1node..."
  }
}'
```

### 9. Get Node Info

Retrieves comprehensive node information.

```json
{
  "node_info": {
    "address": "c4e1node..."
  }
}
```

**Response**:
```json
{
  "address": "c4e1node...",
  "is_whitelisted": true,
  "reputation": 100,
  "added_at": "1698800400000000000",
  "deposit": "100000000",
  "native_staked_amount": "1500000000",
  "tier": 1,
  "last_updated": "1698800400000000000",
  "proof_count": 42,
  "disputed_proofs": 0,
  "unlocking_deposit_amount": null,
  "unlocking_deposit_release_at_block": null
}
```

**Fields**:
- `address` (string): Node address
- `is_whitelisted` (bool): Whether node is registered
- `reputation` (i32): Current reputation score
- `added_at` (optional, Timestamp): Registration timestamp
- `deposit` (optional, Uint128): Current active deposit
- `native_staked_amount` (optional, Uint128): Total native stake (queried from staking module)
- `tier` (optional, u8): Operational tier (1-3, or 0 if non-operational)
- `last_updated` (optional, Timestamp): Last state update
- `proof_count` (optional, u64): Total proofs stored by this node
- `disputed_proofs` (optional, u64): Number of disputed proofs
- `unlocking_deposit_amount` (optional, Uint128): Amount currently unlocking
- `unlocking_deposit_release_at_block` (optional, u64): Block when deposit becomes claimable

**Example**:
```bash
c4ed query wasm contract-state smart <contract_addr> '{
  "node_info": {
    "address": "c4e1node..."
  }
}'
```

## Error Codes

### Admin Errors
- `AdminOnlyOperation`: Operation requires admin privileges
- `Unauthorized`: General authorization failure

### Node Registration Errors
- `NodeAlreadyWhitelisted`: Node already in whitelist
- `NodeNotWhitelisted`: Node not found in registry
- `NodeNotRegistered`: Node not registered (different from not whitelisted)
- `InsufficientStake`: Native stake below minimum for any tier
- `DepositDoesNotMatchTierRequirement`: Sent deposit doesn't match calculated tier requirement

### Deposit Errors
- `DepositAlreadyUnlocking`: Node already has deposit in unbonding period
- `NoDepositToUnlock`: Node has no active deposit
- `NoUnlockedDepositToClaim`: No unlocking deposit entry found
- `DepositNotYetUnlocked`: Unbonding period not complete

### Proof Errors
- `ProofAlreadyExists`: Proof with same hash already stored
- `ProofNotFound`: Proof does not exist
- `InvalidInput`: Generic input validation error
- `InvalidDataOwner`: Data owner address is invalid

### Node Operation Errors
- `InsufficientNodeReputation`: Node reputation below threshold
- `NodeTierNotOperational`: Node tier is 0 (non-operational)
- `NodeHasInsufficientDeposit`: Node's deposit below tier requirement

### System Errors
- `CustomError`: Generic error with custom message
- `StakingQueryError`: Failed to query staking module
- `Std(StdError)`: Standard CosmWasm errors

## Data Types

### Node
```rust
pub struct Node {
    pub address: Addr,
    pub reputation: i32,
    pub added_at: Timestamp,
    pub deposit: Uint128,
    pub tier: u8,                    // 0-3, where 0 is non-operational
    pub proof_count: u64,
    pub disputed_proofs: u64,
    pub last_updated: Timestamp,
}
```

### Proof
```rust
pub struct Proof {
    pub id: u64,
    pub data_hash: String,
    pub original_data_reference: Option<String>,
    pub data_owner: Option<String>,
    pub metadata_json: Option<String>,
    pub stored_at: Timestamp,
    pub stored_by: Addr,
    pub tw_start: Timestamp,         // Time window start
    pub tw_end: Timestamp,           // Time window end
    pub value_in: Option<Uint128>,   // Energy input value
    pub value_out: Option<Uint128>,  // Energy output value
    pub unit: String,                // Measurement unit (e.g., "Wh", "kWh")
}
```

### User
```rust
pub struct User {
    pub address: Addr,
    pub proofs: Vec<u64>,            // List of proof IDs owned by this user
    pub registered_at: Timestamp,
}
```

### UnlockingDeposit
```rust
pub struct UnlockingDeposit {
    pub owner: Addr,
    pub amount: Uint128,
    pub release_at_block: u64,
}
```

## Usage Examples

### Complete Node Registration Workflow

#### 1. Stake Native Tokens (Off-Chain)

```bash
# Stake 5 C4E tokens with a validator
c4ed tx staking delegate c4evaloper1... 5000000000uc4e \
  --from node_operator \
  --gas auto
```

#### 2. Register as Node

```bash
# Qualified for Tier 2 (5 C4E staked)
# Must send 200 C4E deposit (200000000 uc4e)
c4ed tx wasm execute $CONTRACT '{
  "node": {
    "register_node": {}
  }
}' \
  --from node_operator \
  --amount 200000000uc4e \
  --gas auto
```

#### 3. Verify Registration

```bash
c4ed query wasm contract-state smart $CONTRACT '{
  "node_info": {
    "address": "'$(c4ed keys show node_operator -a)'"
  }
}'
```

### Store and Verify Energy Proof

#### 1. Store Proof

```bash
c4ed tx wasm execute $CONTRACT '{
  "node": {
    "store_proof": {
      "data_hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
      "original_data_reference": "ipfs://QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG",
      "data_owner": "c4e1homeowner...",
      "metadata_json": "{\"device_id\":\"meter-001\",\"location\":\"Home-123\",\"verified_by\":\"DeTrack-Worker-01\"}",
      "tw_start": "1698796800000000000",
      "tw_end": "1698800400000000000",
      "value_in": "1500000",
      "value_out": "1200000",
      "unit": "Wh"
    }
  }
}' --from node_operator --gas auto
```

#### 2. Verify Proof Exists

```bash
c4ed tx wasm execute $CONTRACT '{
  "node": {
    "verify_proof": {
      "data_hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
    }
  }
}' --from node_operator --gas auto
```

#### 3. Query Proof Details

```bash
c4ed query wasm contract-state smart $CONTRACT '{
  "proof_by_hash": {
    "data_hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
  }
}'
```

### Deposit Management Workflow

#### 1. Add More Deposit

```bash
c4ed tx wasm execute $CONTRACT '{
  "node": {
    "add_deposit": {}
  }
}' \
  --from node_operator \
  --amount 100000000uc4e \
  --gas auto
```

#### 2. Check Updated Deposit

```bash
c4ed query wasm contract-state smart $CONTRACT '{
  "node_info": {
    "address": "'$(c4ed keys show node_operator -a)'"
  }
}' | jq '.deposit'
```

#### 3. Initiate Withdrawal

```bash
c4ed tx wasm execute $CONTRACT '{
  "node": {
    "unlock_deposit": {}
  }
}' --from node_operator --gas auto
```

#### 4. Check Unlock Status

```bash
c4ed query wasm contract-state smart $CONTRACT '{
  "node_info": {
    "address": "'$(c4ed keys show node_operator -a)'"
  }
}' | jq '{unlocking_amount: .unlocking_deposit_amount, release_at: .unlocking_deposit_release_at_block, current_block: "'$(c4ed query block | jq -r .block.header.height)'"}'
```

#### 5. Claim After Unbonding

```bash
# Wait for release_at_block, then claim
c4ed tx wasm execute $CONTRACT '{
  "node": {
    "claim_unlocked_deposit": {}
  }
}' --from node_operator --gas auto
```

### Admin Operations

#### Update Node Reputation

```bash
c4ed tx wasm execute $CONTRACT '{
  "admin": {
    "update_node_reputation": {
      "node_address": "c4e1node...",
      "reputation": 150
    }
  }
}' --from admin --gas auto
```

#### Configure Treasury

```bash
c4ed tx wasm execute $CONTRACT '{
  "admin": {
    "configure_treasury": {
      "treasury_address": "c4e1treasury..."
    }
  }
}' --from admin --gas auto
```

## Integration Patterns

### 1. DeTrack Worker Integration

```typescript
class DeTrackWorkerClient {
  async registerNode(operatorAddress: string, deposit: Coin): Promise<void> {
    const msg = {
      node: {
        register_node: {}
      }
    };
    
    await this.client.execute(
      operatorAddress,
      this.contractAddress,
      msg,
      "auto",
      undefined,
      [deposit]
    );
  }
  
  async storeProof(
    operatorAddress: string,
    energyData: EnergyData
  ): Promise<number> {
    const msg = {
      node: {
        store_proof: {
          data_hash: energyData.hash,
          original_data_reference: energyData.ipfsCid,
          data_owner: energyData.ownerAddress,
          metadata_json: JSON.stringify(energyData.metadata),
          tw_start: energyData.timeWindowStart,
          tw_end: energyData.timeWindowEnd,
          value_in: energyData.valueIn?.toString(),
          value_out: energyData.valueOut?.toString(),
          unit: energyData.unit
        }
      }
    };
    
    const result = await this.client.execute(
      operatorAddress,
      this.contractAddress,
      msg,
      "auto"
    );
    
    // Extract proof_id from events
    const proofId = this.extractProofId(result);
    return proofId;
  }
}
```

### 2. Energy Data Verification

```typescript
class EnergyVerifier {
  async verifyEnergyProof(dataHash: string): Promise<boolean> {
    try {
      const proof = await this.client.queryContractSmart(
        this.contractAddress,
        { proof_by_hash: { data_hash: dataHash } }
      );
      
      return proof !== null;
    } catch (error) {
      if (error.message.includes("not found")) {
        return false;
      }
      throw error;
    }
  }
  
  async getProofDetails(dataHash: string): Promise<ProofDetails> {
    const proof = await this.client.queryContractSmart(
      this.contractAddress,
      { proof_by_hash: { data_hash: dataHash } }
    );
    
    return {
      id: proof.id,
      hash: proof.data_hash,
      owner: proof.data_owner,
      storedBy: proof.stored_by,
      timeWindow: {
        start: new Date(Number(proof.tw_start) / 1000000),
        end: new Date(Number(proof.tw_end) / 1000000)
      },
      energyValues: {
        input: proof.value_in,
        output: proof.value_out,
        unit: proof.unit
      }
    };
  }
}
```

### 3. Node Monitoring

```typescript
class NodeMonitor {
  async getNodeStatus(nodeAddress: string): Promise<NodeStatus> {
    const nodeInfo = await this.client.queryContractSmart(
      this.contractAddress,
      { node_info: { address: nodeAddress } }
    );
    
    return {
      isRegistered: nodeInfo.is_whitelisted,
      tier: nodeInfo.tier,
      reputation: nodeInfo.reputation,
      depositAmount: nodeInfo.deposit,
      nativeStake: nodeInfo.native_staked_amount,
      proofCount: nodeInfo.proof_count,
      unlockingDeposit: {
        amount: nodeInfo.unlocking_deposit_amount,
        releaseBlock: nodeInfo.unlocking_deposit_release_at_block
      }
    };
  }
  
  async canStoreProofs(nodeAddress: string): Promise<boolean> {
    const config = await this.client.queryContractSmart(
      this.contractAddress,
      { config: {} }
    );
    
    const nodeInfo = await this.getNodeStatus(nodeAddress);
    
    return (
      nodeInfo.isRegistered &&
      nodeInfo.tier > 0 &&
      nodeInfo.reputation >= config.min_reputation_threshold &&
      nodeInfo.depositAmount >= this.getTierDeposit(nodeInfo.tier, config)
    );
  }
}
```

## Best Practices

### For Node Operators

1. **Maintain Sufficient Stake**: Keep native stake above tier threshold
2. **Monitor Reputation**: Track reputation score and address issues promptly
3. **Secure Deposits**: Ensure deposit remains above tier requirement
4. **Regular Proofs**: Consistently store proofs to build reputation
5. **Plan Withdrawals**: Account for 7-day unbonding period

### For Administrators

1. **Regular Monitoring**: Review node performance and reputation
2. **Fair Reputation**: Update reputation based on objective criteria
3. **Treasury Management**: Ensure treasury address is secure
4. **Threshold Tuning**: Adjust reputation threshold based on network health
5. **Documentation**: Document all admin actions for transparency

### For Integrators

1. **Error Handling**: Implement robust error handling for all operations
2. **Gas Estimation**: Always use gas estimation for transactions
3. **Event Monitoring**: Listen for contract events for real-time updates
4. **Retry Logic**: Implement retry with exponential backoff for failed transactions
5. **State Verification**: Query state after transactions to verify success
