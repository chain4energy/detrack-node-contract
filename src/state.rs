use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    /// The administrator of the contract, capable of performing privileged operations.
    pub admin: Addr,
    /// The version of the contract.
    pub version: String,
    /// A counter for the total number of proofs stored, used to assign unique IDs.
    pub proof_count: u64,
    /// The minimum reputation a node must have to perform certain actions (e.g., store proofs).
    pub min_reputation_threshold: i32,
    /// The address of the treasury contract/wallet where slashed funds or fees might be sent.
    pub treasury: Option<Addr>, // Changed from treasury_address
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
}

#[cw_serde]
pub struct Proof {
    /// Unique identifier for the proof.
    pub id: u64,
    /// The hash of the off-chain data, serving as the core content of the proof.
    pub data_hash: String,
    /// An optional reference (e.g., IPFS CID, URL) to the original data.
    pub original_data_reference: Option<String>,
    /// Optional address of the entity that owns or submitted the original data.
    pub data_owner: Option<String>, 
    /// Optional JSON string for additional, application-specific metadata related to the proof.
    pub metadata_json: Option<String>,
    /// Timestamp of when the proof was stored in the contract.
    pub stored_at: Timestamp, // Renamed from verified_at
    /// Address of the node that stored this proof.
    pub stored_by: Addr,
    /// Start of the time window of measurement which the proof pertains to.
    pub tw_start: Timestamp,
    /// End of the time window of measurement which the proof pertains to.
    pub tw_end: Timestamp,
    /// Amount of energy/data produced or input value.
    pub value_in: Option<Uint128>,
    /// Amount of energy/data consumed or output value.
    pub value_out: Option<Uint128>,
    /// Unit for value_in/value_out (e.g., kWh, MWh).
    pub unit: String,
}

#[cw_serde]
pub struct User {
    /// The user's blockchain address.
    pub address: Addr,
    /// A list of IDs of proofs associated with this user, typically as the data owner.
    pub proofs: Vec<u64>,
    /// Timestamp of when the user was first registered in the contract.
    pub registered_at: Timestamp,
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

/// Stores the global configuration of the contract.
pub const CONFIG: Item<Config> = Item::new("config");

/// Stores individual data proofs, keyed by a unique sequential ID (u64).
pub const PROOFS: Map<u64, Proof> = Map::new("proofs");

/// Provides an index to look up a proof ID (u64) by its data hash (String).
/// This allows for quick checks of proof existence and retrieval by content hash.
pub const PROOF_BY_HASH: Map<&str, u64> = Map::new("proof_by_hash");

/// Stores user profiles, keyed by their address (Addr).
/// Users are typically data owners associated with proofs.
pub const USERS: Map<String, User> = Map::new("users");

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