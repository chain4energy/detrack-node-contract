use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Timestamp, Uint128};

/// BatchInfo - Information about a single batch aggregated into a proof
/// Phase 1b: Multi-batch aggregation support
#[cw_serde]
pub struct BatchInfo {
    /// Unique batch identifier (UUID or gateway-generated)
    pub batch_id: String,
    /// W3C DID of gateway that submitted this batch
    pub gateway_did: String,
    /// Number of devices in this batch
    pub device_count: u32,
    /// Total snapshots aggregated in this batch
    pub snapshot_count: u32,
    /// SHA-256 Merkle root of this batch
    pub batch_merkle_root: String,
}

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub version: String,
    /// DID Contract address for identity verification
    pub did_contract_address: String,
    // Add tier and deposit parameters from previous discussion
    pub min_stake_tier1: Uint128,
    pub min_stake_tier2: Uint128,
    pub min_stake_tier3: Uint128,
    pub deposit_tier1: Uint128,
    pub deposit_tier2: Uint128,
    pub deposit_tier3: Uint128,
    pub use_whitelist: bool,
    // Add deposit unlock period parameter
    pub deposit_unlock_period_blocks: u64,
}

/// Message type for admin operations
#[cw_serde]
pub enum AdminExecuteMsg {
    /// Update the admin address
    UpdateAdmin { new_admin: String },
    /// Whitelist a node address
    WhitelistNode { node_address: String },
    /// Remove a node from the whitelist
    RemoveNode { node_address: String },
    /// Update node reputation
    UpdateNodeReputation { 
        node_address: String, 
        #[serde(deserialize_with = "crate::helpers::deserialize_int")]
        reputation: i32 
    },
    /// Update the minimum reputation threshold
    UpdateMinReputationThreshold { 
        #[serde(deserialize_with = "crate::helpers::deserialize_int")]
        threshold: i32 
    },
    /// Configure the treasury address
    ConfigureTreasury { treasury_address: String },
}

/// Message type for node operations
#[cw_serde]
pub enum NodeExecuteMsg {
    /// Store a new proof on the blockchain (Phase 1b: Multi-batch aggregation)
    StoreProof {
        /// W3C DID of the Worker Node storing this proof
        worker_did: String,
        /// SHA-256 hash of the blockchain Merkle root (aggregates all batches)
        data_hash: String,
        /// Start of time window (nanosecond timestamp as string)
        tw_start: String,
        /// End of time window (nanosecond timestamp as string)
        tw_end: String,
        /// Array of batch metadata (one entry per gateway batch)
        batch_metadata: Vec<BatchInfo>,
        /// Optional JSON metadata for additional information
        metadata_json: Option<String>,
    },
    /// Register a new node
    RegisterNode {},
    /// Add to an existing node's deposit
    AddDeposit {}, // Added
    /// Verify a proof
    VerifyProof { data_hash: String },
    /// Initiate unlocking of the node's deposit
    UnlockDeposit {},
    /// Claim unlocked deposit after the unbonding period
    ClaimUnlockedDeposit {},
}

/// Main execute message type that wraps admin and node messages
#[cw_serde]
pub enum ExecuteMsg {
    /// Admin operations
    Admin(AdminExecuteMsg),
    /// Node operations
    Node(NodeExecuteMsg),
}

/// Message type for `migrate` entry_point
#[cw_serde]
pub enum MigrateMsg {
    /// Migrate to new version
    Migrate { new_version: String },
}

/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the current configuration
    #[returns(ConfigResponse)]
    Config {},
    /// Returns a specific proof by ID
    #[returns(ProofResponse)]
    Proof { id: u64 },
    /// Returns a specific proof by data hash
    #[returns(ProofResponse)]
    ProofByHash { data_hash: String },
    /// Returns a list of all proofs
    #[returns(ProofsResponse)]
    Proofs { start_after: Option<u64>, limit: Option<u32> },
    /// Returns a user's profile
    #[returns(UserResponse)]
    User { address: String },
    /// Returns a list of proofs from a specific owner
    #[returns(ProofsResponse)]
    UserProofs { address: String, start_after: Option<u64>, limit: Option<u32> },
    /// Returns whether a node is whitelisted
    #[returns(WhitelistedResponse)]
    IsWhitelisted { address: String },
    /// Returns node reputation
    #[returns(NodeReputationResponse)]
    NodeReputation { address: String },
    /// Returns node information including whitelisted status and reputation
    #[returns(NodeInfoResponse)]
    NodeInfo { address: String },
    /// Returns proofs submitted by a specific Worker Node DID
    #[returns(ProofsResponse)]
    ProofsByWorker { 
        worker_did: String, 
        start_after: Option<u64>, 
        limit: Option<u32> 
    },
    /// Returns proofs that include batches from a specific Gateway DID
    #[returns(ProofsResponse)]
    ProofsByGateway { 
        gateway_did: String, 
        start_after: Option<u64>, 
        limit: Option<u32> 
    },
}

// Query Responses
#[cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub version: String,
    pub proof_count: u64,
    pub min_reputation_threshold: i32,
    pub treasury: Option<String>,
    pub did_contract_address: String,
    // Add fields from InstantiateMsg
    pub min_stake_tier1: Uint128,
    pub min_stake_tier2: Uint128,
    pub min_stake_tier3: Uint128,
    pub deposit_tier1: Uint128,
    pub deposit_tier2: Uint128,
    pub deposit_tier3: Uint128,
    pub use_whitelist: bool,
    pub deposit_unlock_period_blocks: u64,
}

#[cw_serde]
pub struct ProofResponse {
    pub id: u64,
    /// W3C DID of the Worker Node that stored this proof
    pub worker_did: String,
    /// SHA-256 hash of the blockchain Merkle root
    pub data_hash: String,
    /// Start of time window (nanosecond timestamp as string)
    pub tw_start: String,
    /// End of time window (nanosecond timestamp as string)
    pub tw_end: String,
    /// Array of batch metadata (multi-batch aggregation)
    pub batch_metadata: Vec<BatchInfo>,
    /// Optional JSON metadata
    pub metadata_json: Option<String>,
    /// Blockchain timestamp when proof was stored
    pub stored_at: Timestamp,
    /// Address of the node that stored this proof
    pub stored_by: String,
}

#[cw_serde]
pub struct ProofsResponse {
    pub proofs: Vec<ProofResponse>,
}

#[cw_serde]
pub struct UserResponse {
    pub address: String,
    pub proofs: Vec<u64>,
    pub registered_at: Timestamp,
}

#[cw_serde]
pub struct WhitelistedResponse {
    pub is_whitelisted: bool,
}

#[cw_serde]
pub struct NodeReputationResponse {
    pub address: String,
    pub reputation: i32,
}

#[cw_serde]
pub struct NodeInfoResponse {
    pub address: String,
    pub is_whitelisted: bool, // This indicates if the node is in the WHITELISTED_NODES map (i.e., registered)
    pub reputation: i32,
    pub added_at: Option<Timestamp>, // Timestamp of registration or when added by admin
    pub deposit: Option<Uint128>, // Current locked deposit in the contract
    pub native_staked_amount: Option<Uint128>, // Calculated native stake from the staking module
    pub tier: Option<u8>, // Current operational tier
    pub last_updated: Option<Timestamp>, // Last time the node's record was updated
    pub proof_count: Option<u64>,
    pub disputed_proofs: Option<u64>,
    pub unlocking_deposit_amount: Option<Uint128>, // Amount of deposit currently unlocking
    pub unlocking_deposit_release_at_block: Option<u64>, // Block height when the deposit will be claimable
}