# DeTrack Node Contract - Roadmap to Variant C (DID-First Architecture)

**Document Version**: 1.1  
**Date**: 2026-01-13  
**Status**: Phase 1b Complete ✅  
**Current Version**: v0.3.2

---

## 📋 Executive Summary

**Phase 1b COMPLETED** - The DeTrack Node Contract has successfully migrated to **Variant C (DID-First Minimal)** architecture with DID-based trust chain verification, multi-batch proof aggregation, and optimized storage.

**Completed Features (v0.3.2)**:
- ✅ DID-based identity verification (Worker DID, Gateway DID, Device DID)
- ✅ Multi-batch proof aggregation (up to 100 batches per proof)
- ✅ Removed redundant fields (data_owner, value_in/out, unit)
- ✅ Secondary indexes for efficient queries (by_gateway, by_worker)
- ✅ DID Contract integration for authorization
- ✅ Configurable `max_batch_size` parameter (default: 100)
- ✅ 22 comprehensive unit tests passing
- ✅ Timestamp-based storage (optimized vs String)

**Current Deployment**:
- Contract Address: `c4e17p9rzwnnfxcjp32un9ug7yhhzgtkhvl9jfksztgw5uh69wac2pgsn5kv43`
- Code ID: Latest optimized build
- DID Contract: `c4e14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s86dt7n`

**Next Phase**: Integration testing, E2E validation, production deployment

---

## 🎯 Current State (Phase 1a)

### Current ExecuteMsg::StoreProof

```rust
pub enum NodeExecuteMsg {
    StoreProof {
        data_hash: String,                      // SHA-256 hash
        original_data_reference: Option<String>, // IPFS URI, etc.
        data_owner: Option<String>,             // ❌ REMOVE - Redundant
        metadata_json: Option<String>,          // Generic metadata
        tw_start: Timestamp,                    // Time window start
        tw_end: Timestamp,                      // Time window end
        value_in: Option<Uint128>,              // ❌ REMOVE - Meaningless for heterogeneous data
        value_out: Option<Uint128>,             // ❌ REMOVE - Meaningless for heterogeneous data
        unit: String,                           // ❌ REMOVE - Meaningless for heterogeneous data
    },
    // ... other messages
}
```

### Current Storage Structure

```rust
pub struct Proof {
    pub id: u64,
    pub data_hash: String,
    pub original_data_reference: Option<String>,
    pub data_owner: Option<String>,             // ❌ REMOVE
    pub metadata_json: Option<String>,
    pub stored_at: Timestamp,
    pub stored_by: String,                      // Worker address (not DID)
    pub tw_start: Timestamp,
    pub tw_end: Timestamp,
    pub value_in: Option<Uint128>,              // ❌ REMOVE
    pub value_out: Option<Uint128>,             // ❌ REMOVE
    pub unit: String,                           // ❌ REMOVE
}

// Storage: Simple Map, no indexes
pub const PROOFS: Map<u64, Proof> = Map::new("proofs");
```

### Problems with Current Implementation

1. **No Trust Chain**: Cannot verify which Gateway submitted data
2. **No DID Integration**: Worker identity not cryptographically verifiable
3. **Redundant Fields**: `data_owner` duplicates `stored_by`
4. **Meaningless Aggregation**: `value_in/out/unit` cannot aggregate heterogeneous data
5. **No Query Optimization**: Cannot query by Gateway, no secondary indexes
6. **Single-Batch Model**: Each proof = 1 batch (inefficient for multi-gateway)
7. **No Batch Attribution**: Cannot trace which Gateway sent which measurements

---

## 🚀 Target State (Phase 1b - Variant C)

### Target ExecuteMsg::StoreProof

```rust
pub enum NodeExecuteMsg {
    StoreProof {
        worker_did: String,                           // ✅ NEW - Worker DID
        data_hash: String,                            // Aggregated hash (all batches)
        batch_metadata: Vec<BatchInfo>,               // ✅ NEW - Per-batch information
        tw_start: String,                             // Nanoseconds (String for precision)
        tw_end: String,                               // Nanoseconds
        device_ownership_snapshot: Option<Vec<DeviceOwnership>>, // ✅ NEW - Phase 4
        metadata_json: Option<String>,                // Phase 5 flexible metadata
    },
    // ... other messages
}

pub struct BatchInfo {
    pub batch_id: String,                             // ✅ NEW - Unique batch identifier
    pub gateway_did: String,                          // ✅ NEW - Gateway DID
    pub snapshot_count: u32,                          // ✅ NEW - Number of measurements
    pub batch_merkle_root: String,                    // ✅ NEW - Batch Merkle root
}

pub struct DeviceOwnership {                          // Phase 4 only
    pub device_did: String,                           // ✅ NEW - Device DID
    pub nft_id: String,                               // NFT identifier
    pub owner: Addr,                                  // Owner at snapshot time
    pub snapshot_timestamp: String,                   // When ownership captured
}
```

### Target Storage Structure

```rust
pub struct Proof {
    pub id: u64,
    pub worker_did: String,                           // ✅ NEW - Worker DID
    pub data_hash: String,                            // Aggregated hash
    pub batch_metadata: Vec<BatchInfo>,               // ✅ NEW - Batch information
    pub tw_start: String,                             // Nanoseconds
    pub tw_end: String,                               // Nanoseconds
    pub device_ownership_snapshot: Option<Vec<DeviceOwnership>>, // ✅ NEW
    pub metadata_json: Option<String>,
    pub created_at: Timestamp,                        // Blockchain timestamp
}

// Storage: IndexedMap with secondary indexes
pub struct ProofIndexes<'a> {
    pub worker: MultiIndex<'a, String, Proof, u64>,  // ✅ NEW - Index by worker_did
}

pub fn proofs<'a>() -> IndexedMap<'a, u64, Proof, ProofIndexes<'a>> {
    let indexes = ProofIndexes {
        worker: MultiIndex::new(
            |_pk: &[u8], d: &Proof| d.worker_did.clone(),
            "proofs",
            "proofs__worker"
        ),
    };
    IndexedMap::new("proofs", indexes)
}

// Manual gateway index (gateway_did can appear in multiple batches)
pub const GATEWAY_PROOFS: Map<(&str, u64), ()> = Map::new("gateway_proofs");
```

### Benefits of Target State

✅ **Complete Trust Chain**: Gateway → Worker → Blockchain fully traced  
✅ **DID Integration**: W3C-compliant DIDs for all actors  
✅ **Multi-Batch Aggregation**: 8+ batches → 1 blockchain TX (99.94% cost reduction)  
✅ **Query Optimization**: Secondary indexes enable fast lookups  
✅ **No Redundancy**: Removed meaningless/duplicate fields (~210 gas savings)  
✅ **Future-Proof**: Device DID, NFT ownership ready (Phase 3-4)  
✅ **Audit Ready**: Complete provenance trail for compliance

---

## 📅 Implementation Phases

### Phase 1a → Phase 1b Migration (Breaking Change)

**Timeline**: 6-8 days  
**Breaking**: YES - New contract deployment required  
**Rollback**: Deploy old contract version if needed

---

## 🔧 Phase 1b: Implementation Tasks

### Day 1-2: Core Message Structures

**File**: `src/msg.rs`

**Tasks**:
```rust
[ ] Update NodeExecuteMsg::StoreProof
    [ ] Add worker_did: String
    [ ] Add batch_metadata: Vec<BatchInfo>
    [ ] Change tw_start/tw_end to String (nanoseconds)
    [ ] Add device_ownership_snapshot: Option<Vec<DeviceOwnership>>
    [ ] Remove: data_owner, value_in, value_out, unit
    [ ] Remove: original_data_reference (move to metadata_json Phase 5)

[ ] Add BatchInfo struct
    pub struct BatchInfo {
        pub batch_id: String,
        pub gateway_did: String,
        pub snapshot_count: u32,
        pub batch_merkle_root: String,
    }

[ ] Add DeviceOwnership struct (Phase 4 preparation)
    pub struct DeviceOwnership {
        pub device_did: String,
        pub nft_id: String,
        pub owner: Addr,
        pub snapshot_timestamp: String,
    }

[ ] Update ProofResponse
    [ ] Match new Proof structure
    [ ] Remove old fields
    [ ] Add new fields

[ ] Add QueryMsg variants
    [ ] ProofsByWorker { worker_did: String, start_after: Option<u64>, limit: Option<u32> }
    [ ] ProofsByGateway { gateway_did: String, start_after: Option<u64>, limit: Option<u32> }
```

**Validation Rules**:
```rust
[ ] worker_did: Regex match ^did:c4e:worker:[a-zA-Z0-9_-]+$
[ ] gateway_did: Regex match ^did:c4e:gateway:[a-zA-Z0-9_-]+$
[ ] device_did: Regex match ^did:c4e:device:[a-zA-Z0-9_-]+$
[ ] batch_metadata.len() > 0 (at least 1 batch)
[ ] batch_metadata.len() <= 100 (reasonable upper limit)
[ ] tw_start < tw_end
[ ] data_hash: 64 hex characters (SHA-256)
[ ] batch_merkle_root: 64 hex characters (SHA-256)
```

---

### Day 2-3: State Management & Indexes

**File**: `src/state.rs`

**Tasks**:
```rust
[ ] Update Proof struct
    pub struct Proof {
        pub id: u64,
        pub worker_did: String,                    // NEW
        pub data_hash: String,
        pub batch_metadata: Vec<BatchInfo>,        // NEW
        pub tw_start: String,                      // Changed to String
        pub tw_end: String,                        // Changed to String
        pub device_ownership_snapshot: Option<Vec<DeviceOwnership>>, // NEW
        pub metadata_json: Option<String>,
        pub created_at: Timestamp,
    }

[ ] Add ProofIndexes struct
    pub struct ProofIndexes<'a> {
        pub worker: MultiIndex<'a, String, Proof, u64>,
    }

[ ] Implement IndexList for ProofIndexes
    impl<'a> IndexList<Proof> for ProofIndexes<'a> {
        fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Proof>> + '_> {
            let v: Vec<&dyn Index<Proof>> = vec![&self.worker];
            Box::new(v.into_iter())
        }
    }

[ ] Replace PROOFS Map with IndexedMap
    pub fn proofs<'a>() -> IndexedMap<'a, u64, Proof, ProofIndexes<'a>> {
        let indexes = ProofIndexes {
            worker: MultiIndex::new(
                |_pk: &[u8], d: &Proof| d.worker_did.clone(),
                "proofs",
                "proofs__worker"
            ),
        };
        IndexedMap::new("proofs", indexes)
    }

[ ] Add manual gateway index
    pub const GATEWAY_PROOFS: Map<(&str, u64), ()> = Map::new("gateway_proofs");

[ ] Add DID Contract address to Config
    pub struct Config {
        pub admin: Addr,
        pub version: String,
        pub did_contract_address: Addr,        // NEW
        // ... existing fields
    }
```

---

### Day 3-4: Execute Logic & DID Verification

**File**: `src/execute.rs`

**Tasks**:
```rust
[ ] Update execute_store_proof function signature
    pub fn execute_store_proof(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        worker_did: String,
        data_hash: String,
        batch_metadata: Vec<BatchInfo>,
        tw_start: String,
        tw_end: String,
        device_ownership_snapshot: Option<Vec<DeviceOwnership>>,
        metadata_json: Option<String>,
    ) -> Result<Response, ContractError>

[ ] Add DID verification helper
    fn verify_did(
        deps: &Deps,
        did: &str,
    ) -> Result<DidDocument, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        
        // Query DID Contract
        let query_msg = to_json_binary(&did_contract::QueryMsg::GetDidDocument {
            did: did.to_string(),
        })?;
        
        let did_doc: DidDocument = deps.querier.query(&QueryRequest::Wasm(
            WasmQuery::Smart {
                contract_addr: config.did_contract_address.to_string(),
                msg: query_msg,
            }
        ))?;
        
        // Check if DID is active
        if did_doc.deactivated {
            return Err(ContractError::DidDeactivated { did: did.to_string() });
        }
        
        Ok(did_doc)
    }

[ ] Add Worker DID verification
    // Verify worker_did exists and is active
    let worker_did_doc = verify_did(&deps.as_ref(), &worker_did)?;
    
    // Verify msg.sender matches DID controller
    if worker_did_doc.controller != info.sender.to_string() {
        return Err(ContractError::Unauthorized);
    }

[ ] Add Gateway DID verification loop
    // Verify all gateway DIDs
    for batch in &batch_metadata {
        let gateway_did_doc = verify_did(&deps.as_ref(), &batch.gateway_did)?;
        
        // Optional: Additional authorization checks
        // (e.g., verify gateway is linked to this worker)
    }

[ ] Update proof storage
    let proof_id = PROOF_COUNTER.load(deps.storage)?;
    
    let proof = Proof {
        id: proof_id,
        worker_did: worker_did.clone(),
        data_hash: data_hash.clone(),
        batch_metadata: batch_metadata.clone(),
        tw_start: tw_start.clone(),
        tw_end: tw_end.clone(),
        device_ownership_snapshot: device_ownership_snapshot.clone(),
        metadata_json: metadata_json.clone(),
        created_at: env.block.time,
    };
    
    // Save with automatic worker_did indexing
    proofs().save(deps.storage, proof_id, &proof)?;

[ ] Add manual gateway indexing
    // Index by gateway_did (each batch may have different gateway)
    for batch in &batch_metadata {
        GATEWAY_PROOFS.save(
            deps.storage,
            (batch.gateway_did.as_str(), proof_id),
            &()
        )?;
    }

[ ] Update response attributes
    Ok(Response::new()
        .add_attribute("action", "store_proof")
        .add_attribute("proof_id", proof_id.to_string())
        .add_attribute("worker_did", worker_did)
        .add_attribute("batch_count", batch_metadata.len().to_string())
        .add_attribute("data_hash", data_hash))
```

---

### Day 4-5: Query Implementation

**File**: `src/query.rs`

**Tasks**:
```rust
[ ] Update query_proof function
    // Handle new Proof structure
    pub fn query_proof(deps: Deps, id: u64) -> StdResult<ProofResponse> {
        let proof = proofs().load(deps.storage, id)?;
        
        Ok(ProofResponse {
            id: proof.id,
            worker_did: proof.worker_did,
            data_hash: proof.data_hash,
            batch_metadata: proof.batch_metadata,
            tw_start: proof.tw_start,
            tw_end: proof.tw_end,
            device_ownership_snapshot: proof.device_ownership_snapshot,
            metadata_json: proof.metadata_json,
            created_at: proof.created_at,
        })
    }

[ ] Add query_proofs_by_worker
    pub fn query_proofs_by_worker(
        deps: Deps,
        worker_did: String,
        start_after: Option<u64>,
        limit: Option<u32>,
    ) -> StdResult<ProofsResponse> {
        let limit = limit.unwrap_or(50).min(100) as usize;
        let start = start_after.map(Bound::exclusive);
        
        let proofs: Vec<Proof> = proofs()
            .idx
            .worker
            .prefix(worker_did)
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|item| item.map(|(_, proof)| proof))
            .collect::<StdResult<Vec<_>>>()?;
        
        Ok(ProofsResponse { proofs })
    }

[ ] Add query_proofs_by_gateway
    pub fn query_proofs_by_gateway(
        deps: Deps,
        gateway_did: String,
        start_after: Option<u64>,
        limit: Option<u32>,
    ) -> StdResult<ProofsResponse> {
        let limit = limit.unwrap_or(50).min(100) as usize;
        let start = start_after.map(|id| Bound::exclusive((gateway_did.as_str(), id)));
        
        // Get proof IDs from manual index
        let proof_ids: Vec<u64> = GATEWAY_PROOFS
            .prefix(gateway_did.as_str())
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|item| item.map(|(id, _)| id))
            .collect::<StdResult<Vec<_>>>()?;
        
        // Load full proofs
        let proofs: Vec<Proof> = proof_ids
            .iter()
            .filter_map(|id| proofs().load(deps.storage, *id).ok())
            .collect();
        
        Ok(ProofsResponse { proofs })
    }

[ ] Update query_proofs to handle new structure
    // Pagination, filtering, etc.
```

---

### Day 5: Error Handling & Validation

**File**: `src/error.rs`

**Tasks**:
```rust
[ ] Add new error variants
    #[error("DID not found: {did}")]
    DidNotFound { did: String },
    
    #[error("DID deactivated: {did}")]
    DidDeactivated { did: String },
    
    #[error("Invalid DID format: {did}")]
    InvalidDidFormat { did: String },
    
    #[error("Gateway not authorized: {gateway_did}")]
    GatewayNotAuthorized { gateway_did: String },
    
    #[error("Batch metadata is empty")]
    EmptyBatchMetadata,
    
    #[error("Too many batches: {count}, max: {max}")]
    TooManyBatches { count: usize, max: usize },
    
    #[error("Invalid Merkle root format: {root}")]
    InvalidMerkleRoot { root: String },
```

**File**: `src/helpers.rs`

**Tasks**:
```rust
[ ] Add DID format validation
    pub fn validate_did_format(did: &str) -> Result<(), ContractError> {
        // Regex: ^did:c4e:(worker|gateway|device):[a-zA-Z0-9_-]+$
        let re = Regex::new(r"^did:c4e:(worker|gateway|device):[a-zA-Z0-9_-]+$")
            .map_err(|_| ContractError::InvalidDidFormat { did: did.to_string() })?;
        
        if !re.is_match(did) {
            return Err(ContractError::InvalidDidFormat { did: did.to_string() });
        }
        
        Ok(())
    }

[ ] Add hash format validation
    pub fn validate_hash_format(hash: &str) -> Result<(), ContractError> {
        if hash.len() != 64 {
            return Err(ContractError::InvalidHash { hash: hash.to_string() });
        }
        
        if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ContractError::InvalidHash { hash: hash.to_string() });
        }
        
        Ok(())
    }

[ ] Add batch metadata validation
    pub fn validate_batch_metadata(
        batch_metadata: &[BatchInfo]
    ) -> Result<(), ContractError> {
        if batch_metadata.is_empty() {
            return Err(ContractError::EmptyBatchMetadata);
        }
        
        if batch_metadata.len() > 100 {
            return Err(ContractError::TooManyBatches {
                count: batch_metadata.len(),
                max: 100,
            });
        }
        
        for batch in batch_metadata {
            validate_did_format(&batch.gateway_did)?;
            validate_hash_format(&batch.batch_merkle_root)?;
        }
        
        Ok(())
    }
```

---

### Day 6: Testing

**File**: `src/tests.rs` (and integration tests)

**Tasks**:
```rust
[ ] Unit Tests: Message Serialization
    [ ] Test BatchInfo JSON serialization
    [ ] Test DeviceOwnership JSON serialization
    [ ] Test StoreProof message deserialization

[ ] Unit Tests: DID Validation
    [ ] Valid worker DID: did:c4e:worker:detrack1
    [ ] Valid gateway DID: did:c4e:gateway:detrack1
    [ ] Invalid DID (wrong network): did:eth:worker:detrack1
    [ ] Invalid DID (wrong format): worker:detrack1

[ ] Unit Tests: Hash Validation
    [ ] Valid SHA-256 hash (64 hex chars)
    [ ] Invalid hash (wrong length)
    [ ] Invalid hash (non-hex characters)

[ ] Integration Tests: Store Proof
    [ ] Single batch proof
    [ ] Multi-batch proof (8 batches)
    [ ] Multi-gateway proof (3 different gateways)
    [ ] Error: Worker DID not found
    [ ] Error: Gateway DID deactivated
    [ ] Error: msg.sender mismatch
    [ ] Error: Empty batch_metadata
    [ ] Error: Too many batches

[ ] Integration Tests: Query Proofs
    [ ] Query by proof ID
    [ ] Query by worker_did
    [ ] Query by gateway_did
    [ ] Pagination tests (limit, start_after)

[ ] Integration Tests: DID Contract Integration
    [ ] Mock DID Contract responses
    [ ] Test DID verification success
    [ ] Test DID verification failure scenarios

[ ] Gas Cost Measurement
    [ ] Measure gas for single-batch proof
    [ ] Measure gas for 8-batch proof
    [ ] Compare old vs new interface
    [ ] Validate ~210 gas savings per proof
```

---

### Day 7-8: Schema Generation & Documentation

**Tasks**:
```bash
[ ] Regenerate JSON schemas
    cargo run --bin schema
    
[ ] Validate schema files
    ls schema/*.json
    cat schema/execute_msg.json | jq
    cat schema/query_msg.json | jq

[ ] Update InstantiateMsg
    [ ] Add did_contract_address parameter
    [ ] Document requirement: DID Contract must exist

[ ] CLI examples for new interface
    [ ] Store proof with multi-batch
    [ ] Query by worker_did
    [ ] Query by gateway_did
```

---

## 🔄 Migration Strategy

### Pre-Deployment Checklist

```bash
[ ] DID Contract deployed and operational
    [ ] Contract address: c4e1qkphn...
    [ ] Worker DID registered: did:c4e:worker:detrack1
    [ ] Gateway DIDs registered:
        [ ] did:c4e:gateway:detrack1
        [ ] did:c4e:gateway:detrack2
        [ ] did:c4e:gateway:detrack3

[ ] DeTrack Node Contract (new version)
    [ ] Code compiled: cargo build --release --target wasm32-unknown-unknown
    [ ] WASM optimized: rust-optimizer
    [ ] Tests passed: cargo test
    [ ] Schema generated: cargo run --bin schema

[ ] Worker Node (detrack-worker-node)
    [ ] Updated to new interface
    [ ] WORKER_DID configured
    [ ] DID_CONTRACT_ADDRESS configured
    [ ] Batch aggregation logic implemented
    [ ] Tests passed
```

### Deployment Steps

```bash
# 1. Store new contract code
CODE_ID=$(c4ed tx wasm store detrack-node-contract.wasm \
  --from deployer \
  --gas auto \
  --gas-adjustment 1.5 \
  -y | jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')

# 2. Instantiate new contract
c4ed tx wasm instantiate $CODE_ID \
  '{
    "admin": null,
    "version": "1.0.0-phase1b",
    "did_contract_address": "c4e1qkphn8h2rnyqjjtfh8j8dtuqgh5cac57nq2286tsljducqp4lwfqvsysy0",
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
  --label "detrack-node-phase1b" \
  --gas auto \
  --gas-adjustment 1.5 \
  --no-admin

# 3. Update Worker Node configuration
# Edit detrack-worker-node/config/.env
CONTRACT_ADDRESS="c4e1newcontractaddress..."
WORKER_DID="did:c4e:worker:detrack1"
DID_CONTRACT_ADDRESS="c4e1qkphn..."

# 4. Restart Worker Node
pm2 restart detrack-worker-node

# 5. Verify deployment
c4ed query wasm contract-state smart $CONTRACT_ADDRESS '{"config": {}}'

# 6. Submit test proof (multi-batch)
# (Automated via Worker Node batch aggregation)
```

### Data Migration

**Note**: Phase 1a → Phase 1b is a **breaking change** with **no data migration**.

**Rationale**:
- Old and new structures are incompatible
- Historical proofs remain on old contract (immutable)
- New proofs use new contract
- Both contracts can coexist (different addresses)

**If historical data needed**:
```bash
# Export from old contract (manual process)
c4ed query wasm contract-state all $OLD_CONTRACT | jq '.models[] | select(.key | contains("proof"))'

# No automated migration tool (data structures too different)
```

### Rollback Plan

**If deployment fails**:
```bash
# 1. Revert Worker Node to old contract address
CONTRACT_ADDRESS="c4e1oldcontractaddress..."

# 2. Restart Worker Node
pm2 restart detrack-worker-node

# 3. Old contract still operational (unchanged)
```

**Permanent rollback** (if Phase 1b abandoned):
- Keep using Phase 1a contract
- Discard Phase 1b contract (no state, just deployed code)

---

## 📊 Success Metrics

### Functional Metrics

```bash
[ ] DID verification working
    [ ] Worker DID verified on proof submission
    [ ] Gateway DIDs verified for all batches
    [ ] Unauthorized submissions rejected

[ ] Multi-batch aggregation working
    [ ] 8 batches aggregated into single proof
    [ ] All batch_metadata correctly stored
    [ ] batch_merkle_roots preserved

[ ] Query optimization working
    [ ] query_proofs_by_worker returns correct results
    [ ] query_proofs_by_gateway returns correct results
    [ ] Pagination working correctly

[ ] Trust chain complete
    [ ] Can trace Gateway → Worker → Blockchain
    [ ] DID Contract queries successful
    [ ] Complete audit trail preserved
```

### Performance Metrics

```bash
[ ] Gas cost optimization
    [ ] ~210 gas saved per proof (removed fields)
    [ ] Multi-batch aggregation: 99.94% cost reduction
    [ ] Query by DID: < 100ms response time

[ ] Scalability
    [ ] Handle 100+ gateways
    [ ] Support 8-16 batches per proof
    [ ] Indexing performance acceptable
```

### Quality Metrics

```bash
[ ] Test coverage
    [ ] Unit tests: > 80% coverage
    [ ] Integration tests: All scenarios covered
    [ ] End-to-end tests: Multi-gateway flow tested

[ ] Documentation complete
    [ ] API specification updated (this roadmap)
    [ ] CLI examples provided
    [ ] Migration guide documented
```

---

## 🔮 Future Phases

### Phase 2: Multi-Gateway Aggregation at Scale

**Timeline**: 2-3 weeks  
**Dependencies**: Phase 1b complete

**Features**:
- 3-level Merkle tree (measurements → batches → multi-gateway → blockchain)
- 100+ gateways aggregated into single proof
- Worker Node batch scheduling optimization
- Performance tuning for large-scale aggregation

**Changes to Smart Contract**: Minimal (architecture already supports)

---

### Phase 3: Device DID Integration

**Timeline**: 2-3 weeks  
**Dependencies**: Phase 2 complete

**Features**:
- Device-level DID registration
- Measurements attributed to specific device DIDs
- Device authorization via DID Contract

**Changes to Smart Contract**:
```rust
// Update MeasurementSnapshot (Worker Node side)
pub struct MeasurementSnapshot {
    pub device_did: String,  // NEW - Replace device_id
    // ... existing fields
}

// Smart Contract: Query device DIDs from proofs
pub fn query_devices_in_proof(
    deps: Deps,
    proof_id: u64,
) -> StdResult<Vec<String>> {
    // Return unique device DIDs from proof
}
```

---

### Phase 4: NFT Ownership Snapshots

**Timeline**: 2-3 weeks  
**Dependencies**: Phase 3 complete, linkage-contract deployed

**Features**:
- Immutable NFT ownership snapshots at measurement time
- Link to linkage-contract for NFT data
- Historical ownership audit trail

**Changes to Smart Contract**:
```rust
// Already prepared in Phase 1b!
pub struct DeviceOwnership {
    pub device_did: String,
    pub nft_id: String,
    pub owner: Addr,
    pub snapshot_timestamp: String,
}

// Just activate in Phase 4:
device_ownership_snapshot: Some(ownership_data)
```

---

### Phase 5: Flexible Metadata & Analytics

**Timeline**: 1-2 weeks  
**Dependencies**: Phase 4 complete

**Features**:
- Flexible JSON metadata for application-specific data
- Off-chain analytics API
- Custom aggregations and reports

**Changes to Smart Contract**: None (already supported via `metadata_json`)

**Worker Node Changes**:
```rust
// Enable metadata_json population
metadata_json: Some(r#"{
  "aggregation_stats": {
    "total_devices": 157,
    "device_types": ["ac-meter", "solar", "water"],
    "measurement_period_hours": 8
  },
  "quality_metrics": {
    "data_completeness": 0.987,
    "outliers_detected": 3
  }
}"#)
```

---

## 📚 References

### Internal Documentation

- [STORE-PROOF-ANALYSIS.md](../../detrack-worker-node/docs/STORE-PROOF-ANALYSIS.md) - Variant C architecture analysis
- [DeTrack-Network-HLD.md](../../detrack-worker-node/docs/DeTrack-Network-HLD.md) - System architecture
- [ADR-003](../../detrack-worker-node/docs/ADR/ADR-003-remove-aggregation-fields.md) - Remove aggregation fields rationale
- [ADR-004](../../detrack-worker-node/docs/ADR/ADR-004-did-integration.md) - DID integration design

### External Standards

- [W3C DID Specification](https://www.w3.org/TR/did-core/) - DID standard
- [CosmWASM Documentation](https://docs.cosmwasm.com/) - Smart contract framework
- [cw-storage-plus](https://crates.io/crates/cw-storage-plus) - Indexed storage

### Related Contracts

- **did-contract**: `/home/greg/projects/c4e/did-contract`
- **linkage-contract**: `/home/greg/projects/c4e/linkage-contract` (Phase 4)
- **escrow-contract**: `/home/greg/projects/c4e/escrow-contract` (future)

---

## ✅ Approval & Sign-Off

**Technical Lead**: [ ] Approved  
**Security Team**: [ ] Approved  
**Product Owner**: [ ] Approved  

**Deployment Authorization**: [ ] Authorized for Phase 1b

---

**Last Updated**: 2026-01-01  
**Version**: 1.0  
**Status**: Ready for Implementation
