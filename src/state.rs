use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map, IndexedMap, MultiIndex, Index, IndexList};
use crate::msg::BatchInfo;

#[cw_serde]
pub struct Config {
    /// The administrator of the contract, capable of performing privileged operations.
    pub admin: Addr,
    /// A counter for the total number of proofs stored, used to assign unique IDs.
    pub proof_count: u64,
    /// The minimum reputation a node must have to perform certain actions (e.g., store proofs).
    pub min_reputation_threshold: i32,
    /// The address of the treasury contract/wallet where slashed funds or fees might be sent.
    pub treasury: Option<Addr>,
    /// The address of the DID Contract for identity verification
    pub did_contract_address: Addr,
    /// Minimum native stake required for a node to qualify for Tier 1.
    pub min_stake_tier1: Uint128,
    /// Minimum native stake required for a node to qualify for Tier 2.
    pub min_stake_tier2: Uint128,
    /// Minimum native stake required for a node to qualify for Tier 3.
    pub min_stake_tier3: Uint128,
    /// The amount of contract-locked deposit required for a Tier 1 node (in the chain's native staking denomination).
    pub deposit_tier1: Uint128,
    /// The amount of contract-locked deposit required for a Tier 2 node.
    pub deposit_tier2: Uint128,
    /// The amount of contract-locked deposit required for a Tier 3 node.
    pub deposit_tier3: Uint128,
    /// If true, nodes must be explicitly whitelisted by the admin to register or operate.
    /// If false, nodes can register directly by meeting stake/deposit requirements.
    pub use_whitelist: bool,
    /// The duration in blocks for which a node's deposit remains locked after initiating an unlock, before it can be claimed.
    pub deposit_unlock_period_blocks: u64,
    /// The maximum batch size (in number of snapshots) that a node can submit in a single proof.
    /// This helps prevent excessively large proofs that could strain contract resources.
    pub max_batch_size: u32,
}

#[cw_serde]
pub struct Proof {
    /// Unique identifier for the proof.
    pub id: u64,
    /// W3C DID of the Worker Node that stored this proof
    pub worker_did: String,
    /// The hash of the blockchain Merkle root (aggregates all batches)
    pub data_hash: String,
    /// Start of time window (CosmWasm Timestamp)
    pub tw_start: Timestamp,
    /// End of time window (CosmWasm Timestamp)
    pub tw_end: Timestamp,
    /// Timestamp of when the proof was stored in the contract.
    pub stored_at: Timestamp,
    /// Address of the node that stored this proof.
    pub stored_by: Addr,

    /// Array of batch metadata (multi-batch aggregation)
    pub batch_metadata: Vec<BatchInfo>,
    /// Optional reference (e.g., IPFS CID or URI) to the original full data used to generate the proof.
    pub original_data_reference: Option<String>,
    /// Optional JSON string for additional, application-specific metadata related to the proof.
    pub metadata_json: Option<String>,
}

#[cw_serde]
pub struct Node {
    /// The node's blockchain address.
    pub address: Addr,
    /// The node's reputation score, influencing its ability to perform actions.
    pub reputation: i32,
    /// Timestamp of when the node was added or successfully registered.
    pub added_at: Timestamp,
    /// The amount of tokens currently locked as an active deposit by the node in the contract.
    /// This deposit is in the chain's native staking denomination (e.g., "uc4e").
    pub deposit: Uint128,
    /// The operational tier of the node (1, 2, or 3), determined by their native stake.
    pub tier: u8,
    /// Number of proofs successfully stored by this node.
    pub proof_count: u64,
    /// Number of proofs from this node that have been disputed.
    /// // TODO: Implement dispute mechanism and link this to slashing logic.
    pub disputed_proofs: u64,
    /// Timestamp of the last update to any field in this node's record.
    pub last_updated: Timestamp,
}

#[cw_serde]
pub struct UnlockingDeposit {
    /// The address of the node whose deposit is currently in the unbonding/unlocking period.
    pub owner: Addr,
    /// The amount and denomination of the deposit being unlocked.
    pub amount: Uint128, // Ensure this is Uint128
    /// The block height at which this deposit becomes claimable by the owner.
    pub release_at_block: u64,
}

// ============================================================================
// Storage Structures
// ============================================================================

/// Stores the global configuration of the contract.
pub const CONFIG: Item<Config> = Item::new("config");

/// Phase 1b: IndexedMap with secondary indexes for efficient querying
/// ProofIndexes enables querying proofs by worker_did
pub struct ProofIndexes<'a> {
    /// Index by worker_did for efficient Worker Node queries
    pub worker: MultiIndex<'a, String, Proof, u64>,
}

impl<'a> IndexList<Proof> for ProofIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Proof>> + '_> {
        let v: Vec<&dyn Index<Proof>> = vec![&self.worker];
        Box::new(v.into_iter())
    }
}

/// Stores individual data proofs with secondary indexes
/// Primary key: u64 (proof ID)
/// Secondary index: worker_did (String)
pub fn proofs<'a>() -> IndexedMap<'a, u64, Proof, ProofIndexes<'a>> {
    let indexes = ProofIndexes {
        worker: MultiIndex::new(
            |_pk, d| d.worker_did.clone(),
            "proofs",
            "proofs__worker"
        ),
    };
    IndexedMap::new("proofs", indexes)
}

/// Manual index for gateway_did (since multiple batches can have different gateways)
/// Key: (gateway_did, proof_id)
/// Value: () - just for membership checking
pub const GATEWAY_PROOFS: Map<(&str, u64), ()> = Map::new("gateway_proofs");

/// Provides an index to look up a proof ID (u64) by its data hash (String).
/// This allows for quick checks of proof existence and retrieval by content hash.
pub const PROOF_BY_HASH: Map<&str, u64> = Map::new("proof_by_hash");

/// Stores information about registered nodes, keyed by their address (Addr).
/// This is the primary registry for active nodes in the system.
pub const NODES: Map<&Addr, Node> = Map::new("nodes");

/// If `use_whitelist` in Config is true, this map stores addresses explicitly whitelisted by an admin.
/// Being in this list might be a prerequisite for node registration or certain operations.
/// The value is a boolean, typically true if the address is whitelisted.
pub const WHITELISTED_NODES: Map<String, Node> = Map::new("whitelisted_nodes");

/// Stores information about node deposits that are currently in the unbonding/unlocking period.
/// Keyed by the node's address (Addr).
pub const UNLOCKING_DEPOSITS: Map<String, UnlockingDeposit> = Map::new("unlocking_deposits");