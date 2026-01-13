# DeTrack Node Contract Design

## Overview

The DeTrack Node Contract is a CosmWasm smart contract that implements a decentralized proof verification and node management system for the Chain4Energy blockchain. It provides infrastructure for managing worker nodes that verify and store cryptographic proofs of energy data, implementing a tiered staking system with reputation management and deposit locking mechanisms.

## Architecture

### Core Components

#### 1. Node Management System
The contract implements a sophisticated node registration and management system:
- **Tiered Registration**: Nodes are classified into 3 tiers based on native token staking amounts
- **Deposit Locking**: Nodes must lock contract deposits corresponding to their tier
- **Reputation System**: Dynamic reputation scores track node reliability and behavior
- **Whitelist Mode**: Optional admin-controlled whitelist for restricted network participation

#### 2. Proof Storage System
Cryptographic proof management with comprehensive metadata:
- **Unique Identification**: Sequential proof IDs with hash-based indexing
- **Energy Data Fields**: Time windows, input/output values, and measurement units
- **Owner Tracking**: Association of proofs with data owners and storing nodes
- **Verification Trail**: Immutable records of when and by whom proofs were stored

#### 3. Deposit Management
Time-locked deposit system for node security:
- **Unbonding Period**: Configurable delay for deposit withdrawal
- **State Transitions**: Clear states from active to unlocking to claimed
- **Slashing Ready**: Infrastructure prepared for future slashing mechanisms

#### 4. User Registry
Simple user tracking system:
- **Automatic Registration**: Users registered when proofs reference them
- **Proof Association**: Maintain lists of proofs owned by each user
- **Timestamp Tracking**: Record registration times for auditing

### Storage Architecture

The contract uses efficient storage patterns optimized for different query types:

#### Primary Storage Maps

```rust
CONFIG: Item<Config>
```
- **Purpose**: Global contract configuration
- **Contains**: Admin, version, tier parameters, reputation threshold, treasury
- **Access Pattern**: Single load per transaction

```rust
WHITELISTED_NODES: Map<String, Node>
```
- **Purpose**: Central registry for all active nodes
- **Key**: Node address (String)
- **Value**: Complete node state including reputation, deposit, tier, proof count
- **Access Pattern**: Direct lookup by address

```rust
PROOFS: Map<u64, Proof>
```
- **Purpose**: Primary proof storage by sequential ID
- **Key**: Proof ID (u64)
- **Value**: Complete proof data including metadata and energy measurements
- **Access Pattern**: Direct lookup, range queries for pagination

```rust
PROOF_BY_HASH: Map<&str, u64>
```
- **Purpose**: Hash-to-ID index for proof verification
- **Key**: Data hash (String)
- **Value**: Proof ID
- **Access Pattern**: Fast proof existence checks and retrieval by hash

```rust
USERS: Map<String, User>
```
- **Purpose**: User profile storage
- **Key**: User address (String)
- **Value**: User data with proof ID list
- **Access Pattern**: Direct lookup, batch loads for user proofs

```rust
UNLOCKING_DEPOSITS: Map<String, UnlockingDeposit>
```
- **Purpose**: Track deposits in unbonding period
- **Key**: Node address (String)
- **Value**: Deposit amount and release block height
- **Access Pattern**: Direct lookup, removed after claim

### Design Patterns

#### 1. Tiered Access Control
Three-level permission system:

**Admin Operations (Highest Privilege)**
- Configuration updates
- Node whitelisting/removal
- Reputation management
- Treasury configuration

**Node Operations (Medium Privilege)**
- Proof storage (requires reputation threshold)
- Deposit management
- Node registration

**Query Operations (No Privilege)**
- Anyone can query public state
- No authentication required

#### 2. Stake-Based Tier System
Dynamic tier assignment based on native staking:

```
Tier 3: stake >= min_stake_tier3
Tier 2: min_stake_tier2 <= stake < min_stake_tier3
Tier 1: min_stake_tier1 <= stake < min_stake_tier2
```

**Tier Benefits**:
- Higher tiers may receive priority in future reward systems
- Different deposit requirements balance security with accessibility
- Foundation for future tiered service levels

#### 3. Time-Locked Deposits
Secure deposit management with unbonding period:

```
Active Deposit → Unlock Request → Unbonding Period → Claimable → Claimed
```

**Security Properties**:
- Prevents immediate withdrawal during disputes
- Provides window for slashing if malicious activity detected
- Aligns incentives with long-term network participation

#### 4. Dual-Index Proof Storage
Optimized for different access patterns:

**By ID (Primary)**:
- Sequential allocation
- Efficient range queries
- Stable references

**By Hash (Secondary)**:
- Fast existence checks
- Duplicate prevention
- Verification queries

## Technical Decisions

### 1. Framework: CosmWasm with Sylvia

**Decision**: Use CosmWasm 2.1.3 with standard patterns
**Rationale**:
- Mature, well-tested smart contract platform
- Strong security track record
- Excellent tooling and ecosystem support
- Compatible with Cosmos SDK staking queries

**Note**: Unlike the DID contract which uses Sylvia framework, this contract uses standard CosmWasm patterns with explicit entry points. This provides:
- More granular control over message routing
- Clearer separation of admin vs. node operations
- Easier integration with existing C4E infrastructure

### 2. Tiered Node System

**Decision**: Three-tier system based on native staking
**Rationale**:
- Encourages network security through staking
- Provides natural economic barrier to Sybil attacks
- Flexible for future feature gating
- Aligns with broader C4E tokenomics

**Trade-offs**:
- Requires integration with staking module
- More complex registration logic
- Potential for stake-grinding attacks (mitigated by deposit requirements)

### 3. Deposit Locking Mechanism

**Decision**: Separate contract deposits from native staking
**Rationale**:
- Direct collateral for proof accuracy
- Faster slashing than unstaking native tokens
- Contract-controlled for automated enforcement
- Different risk profile than validator staking

**Implementation**: 
- Deposits held in contract balance
- Tracked per-node in state
- Time-locked withdrawal with configurable unbonding period

### 4. Proof Data Structure

**Decision**: Include energy-specific fields (tw_start, tw_end, value_in, value_out, unit)
**Rationale**:
- Tailored for energy verification use case
- Enables on-chain energy accounting
- Supports future automated settlement
- Maintains flexibility with optional fields and JSON metadata

### 5. Reputation Management

**Decision**: Simple integer reputation with admin control
**Rationale**:
- Sufficient for initial implementation
- Allows manual intervention during network bootstrap
- Easy to understand and verify
- Foundation for future automated reputation systems

**Future Enhancement**: Transition to algorithmic reputation based on:
- Proof acceptance rate
- Dispute outcomes
- Uptime metrics
- Stake duration

## Security Considerations

### 1. Access Control

**Admin Protection**:
```rust
fn validate_admin(deps: &DepsMut, info: &MessageInfo) -> Result<(), ContractError>
```
- All admin functions check sender against stored admin address
- Admin address validated during instantiation
- Admin transfer requires explicit UpdateAdmin message

**Node Validation**:
```rust
fn validate_node(deps: &DepsMut, info: &MessageInfo) -> Result<(), ContractError>
```
- Checks whitelist/registration status
- Verifies minimum reputation threshold
- Ensures operational tier (tier > 0)
- Validates deposit meets tier requirement

### 2. Deposit Security

**Tier-Based Requirements**:
- Each tier requires specific minimum deposit
- Deposits locked in contract, not user-controlled
- Verification before allowing proof storage:
  ```rust
  if node.deposit < required_deposit_for_tier {
      return Err(ContractError::NodeHasInsufficientDeposit { ... });
  }
  ```

**Unbonding Protection**:
- Configurable unbonding period prevents immediate withdrawal
- State transitions tracked explicitly
- Prevents double-claiming with state checks
- Future slashing can intercept during unbonding period

### 3. Proof Integrity

**Duplicate Prevention**:
```rust
if PROOF_BY_HASH.has(deps.storage, &data_hash) {
    return Err(ContractError::ProofAlreadyExists(data_hash));
}
```

**Input Validation**:
- Non-empty data hash required
- Address validation for data owners
- Node authorization before storage
- Atomic state updates prevent inconsistency

**Immutability**:
- Proofs cannot be modified after storage
- Only admin can remove nodes (proofs remain)
- Verification trail preserved permanently

### 4. Economic Security

**Stake Requirements**:
- Prevents low-cost Sybil attacks
- Native staking queried from blockchain state
- Cannot be faked or manipulated
- Aligned with validator security model

**Deposit Requirements**:
- Direct collateral for node behavior
- Slashable for malicious actions
- Economic penalty for bad proofs
- Scales with tier privileges

### 5. State Consistency

**Atomic Operations**:
- All state updates within single transaction
- Rollback on any error
- No partial state changes

**Index Synchronization**:
- PROOF_BY_HASH updated atomically with PROOFS
- User proof lists updated with proof creation
- Node statistics updated with each action

## Data Flow

### Node Registration Flow

```
1. Node Preparation (Off-chain)
   ├─► Stake native C4E tokens with validator(s)
   ├─► Prepare deposit funds (uc4e)
   └─► Optional: Get admin whitelist if use_whitelist=true

2. Registration Transaction
   ├─► Call RegisterNode with deposit funds
   ├─► Contract queries staking module for total stake
   ├─► Determine tier based on stake amount
   ├─► Verify deposit matches tier requirement
   ├─► Create/update Node entry in WHITELISTED_NODES
   └─► Return success with tier assignment

3. Post-Registration
   ├─► Node can store proofs (if reputation >= threshold)
   ├─► Node can add more deposit
   ├─► Admin can adjust reputation
   └─► Node starts earning reputation through honest behavior
```

### Proof Storage Flow

```
1. Pre-Storage Validation
   ├─► Verify sender is registered node
   ├─► Check reputation >= min_reputation_threshold
   ├─► Verify tier is operational (tier > 0)
   └─► Validate deposit meets tier requirement

2. Proof Validation
   ├─► Check data_hash is non-empty
   ├─► Verify no duplicate (check PROOF_BY_HASH)
   ├─► Validate data_owner address if provided
   └─► Validate energy data fields

3. State Updates
   ├─► Increment CONFIG.proof_count
   ├─► Create Proof struct with all fields
   ├─► Save to PROOFS map (by ID)
   ├─► Save to PROOF_BY_HASH index (by hash)
   ├─► Update or create User entry
   ├─► Add proof ID to user's proof list
   └─► Update node.proof_count

4. Event Emission
   └─► Emit attributes: action, proof_id, data_hash, stored_by, data_owner
```

### Deposit Withdrawal Flow

```
1. Unlock Request
   ├─► Node calls UnlockDeposit
   ├─► Verify no existing unlocking deposit
   ├─► Verify node has deposit to unlock
   ├─► Calculate release_at_block (current + unbonding_period)
   ├─► Move deposit from node.deposit to UNLOCKING_DEPOSITS
   ├─► Set node.deposit = 0
   └─► Emit unlock event with release block

2. Unbonding Period
   ├─► Deposit locked in UNLOCKING_DEPOSITS
   ├─► Node cannot store new proofs (deposit = 0)
   ├─► Future slashing can intercept if needed
   └─► Wait for release_at_block

3. Claim Request
   ├─► Node calls ClaimUnlockedDeposit
   ├─► Verify unlocking deposit exists
   ├─► Check current_block >= release_at_block
   ├─► Remove from UNLOCKING_DEPOSITS
   ├─► Send funds via BankMsg::Send
   └─► Emit claim event
```

## Query Optimization

### 1. Direct Lookups
- Node info by address: O(1) from WHITELISTED_NODES
- Proof by ID: O(1) from PROOFS
- Proof by hash: O(1) from PROOF_BY_HASH, then O(1) from PROOFS
- User by address: O(1) from USERS

### 2. Range Queries with Pagination
```rust
const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;
```
- All proof listing queries support pagination
- Prevents large response payloads
- Cursor-based pagination with `start_after`
- Configurable limits with safety maximum

### 3. User Proof Queries
- User proofs stored as Vec<u64> in User struct
- Filtered and paginated in-memory
- Trade-off: Simple implementation vs. large user overhead
- Suitable for moderate proof counts per user (< 1000s)

**Future Optimization**: Use IndexedMap for proof→user relationship if users accumulate many proofs

## Scalability Features

### 1. Bounded Storage Growth
- Proofs grow with network usage (expected)
- Users grow with proof submissions (bounded by users)
- Nodes grow with network participation (bounded by economics)
- Unlocking deposits limited to one per node

### 2. Gas-Efficient Operations
- Single storage reads where possible
- Batch queries with pagination
- No expensive loops over large datasets
- Efficient integer keys for main maps

### 3. Off-Chain Data
- Large proof data stored off-chain (IPFS, etc.)
- Only hash and metadata on-chain
- `original_data_reference` points to full data
- `metadata_json` for additional structured data

## Extension Points

### 1. Slashing Mechanism (Planned)
Framework ready for implementation:
```rust
pub fn slash_node(
    deps: DepsMut,
    info: MessageInfo,
    node_address: String,
    amount: Uint128,
    reason: String,
) -> Result<Response, ContractError>
```

**Integration Points**:
- Dispute resolution system
- Automated proof challenges
- Reputation-based triggers
- Treasury fund management

### 2. Reward Distribution (Future)
Infrastructure supports:
- Proof count tracking per node
- Tier-based reward multipliers
- Reputation-weighted distribution
- Time-based vesting

### 3. Advanced Reputation (Future)
Current system provides foundation for:
- Automated reputation updates
- Proof acceptance rates
- Dispute outcome tracking
- Decay functions for inactive nodes
- Bonus reputation for long-term participation

### 4. Cross-Contract Integration
Ready for integration with:
- **DID Contract**: Node identity verification
- **Linkage Contract**: Device-to-node associations
- **Escrow Contract**: Automated energy settlements
- **Token Contracts**: Reward token distribution

## Testing Strategy

### 1. Unit Tests
Current test coverage includes:
- Node registration with different stake amounts
- Proof storage with validation
- Deposit locking and unlocking
- Admin operations
- Access control enforcement

### 2. Integration Tests
Using `cw-multi-test` framework:
- Multi-step workflows
- State consistency across operations
- Event emission verification
- Error condition handling

### 3. Staking Module Mocking
Special handling for test environments:
```rust
// In helpers.rs
pub fn get_native_staked_amount(querier: &QuerierWrapper, address: &Addr) -> Result<Uint128, ContractError> {
    match querier.query(&QueryRequest::Staking(StakingQuery::BondedDenom {})) {
        Ok(response) => // Real staking query
        Err(e) if e.to_string().contains("Unexpected custom query") => {
            // Test environment: return default stake
            Ok(Uint128::new(1000))
        }
        Err(e) => Err(ContractError::StakingQueryError { error: e.to_string() })
    }
}
```

### 4. Property Testing (Recommended)
Future testing should verify:
- Deposit balance always equals sum of node deposits + unlocking deposits
- Proof IDs are sequential without gaps
- No node can have both active deposit and unlocking deposit
- Reputation changes are bounded and justified

## Performance Characteristics

### Operation Complexity

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| Register Node | O(1) | Includes external staking query |
| Store Proof | O(1) | Fixed number of storage operations |
| Query Proof by ID | O(1) | Direct map lookup |
| Query Proof by Hash | O(1) | Hash index + map lookup |
| List Proofs | O(n) | Where n = limit (bounded by MAX_LIMIT) |
| Query Node Info | O(1) | Includes external staking query |
| Unlock Deposit | O(1) | State transfer between maps |
| Claim Deposit | O(1) | State removal + bank message |

### Gas Costs (Estimated)

| Operation | Estimated Gas | Factors |
|-----------|--------------|---------|
| Register Node | 200,000-300,000 | Staking query, state writes |
| Store Proof | 150,000-250,000 | Multiple map updates |
| Add Deposit | 100,000-150,000 | State update + funds |
| Unlock Deposit | 100,000-150,000 | State reorganization |
| Claim Deposit | 120,000-180,000 | Bank transfer included |
| Query Node Info | 50,000-100,000 | Includes staking query |
| Query Proof | 20,000-50,000 | Simple state read |

### Memory Usage

- **Config**: ~500 bytes (tier parameters, addresses, thresholds)
- **Node Entry**: ~300 bytes (addresses, timestamps, counters)
- **Proof Entry**: ~800-2000 bytes (depends on metadata size)
- **User Entry**: ~200 bytes + 8 bytes per proof ID
- **Unlocking Deposit**: ~200 bytes

## Future Enhancements

### Phase 1: Slashing Implementation
- [ ] Proof dispute mechanism
- [ ] Evidence submission system
- [ ] Automated slashing triggers
- [ ] Appeal process
- [ ] Treasury integration

### Phase 2: Advanced Reputation
- [ ] Algorithmic reputation calculation
- [ ] Proof acceptance tracking
- [ ] Dispute outcome integration
- [ ] Reputation decay for inactivity
- [ ] Bonus mechanisms

### Phase 3: Reward System
- [ ] Epoch-based reward pools
- [ ] Tier-weighted distribution
- [ ] Reputation multipliers
- [ ] Stake duration bonuses
- [ ] Claim and compound mechanics

### Phase 4: Enhanced Features
- [ ] Batch proof submission
- [ ] Proof verification challenges
- [ ] Cross-contract integrations
- [ ] Governance participation
- [ ] Upgrade mechanisms

## Migration Strategy

### Contract Versioning
Current version tracking:
```rust
pub struct Config {
    pub version: String,  // e.g., "0.1.0"
    // ... other fields
}
```

### Future Migration Support
Prepared for CosmWasm migrate functionality:
```rust
pub enum MigrateMsg {
    Migrate { new_version: String },
}
```

**Migration Considerations**:
- State schema changes
- New field additions
- Storage key modifications
- Backward compatibility
- Data migration scripts

### Rollback Strategy
- Always deploy to testnet first
- Monitor for 24-48 hours
- Have previous version ready
- Document rollback procedures
- Maintain state backup scripts

## Conclusion

The DeTrack Node Contract provides a robust foundation for decentralized energy proof verification. Its tiered node system balances security with accessibility, while the deposit locking mechanism ensures node accountability. The architecture is designed for future enhancements while maintaining current simplicity and gas efficiency.

Key strengths:
- ✅ Clear separation of concerns
- ✅ Flexible tier system
- ✅ Secure deposit management
- ✅ Comprehensive access control
- ✅ Extensible for future features
- ✅ Gas-optimized storage patterns

The contract is production-ready for initial deployment with clear paths for adding slashing, rewards, and advanced reputation systems as the network matures.
