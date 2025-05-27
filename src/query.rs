use cosmwasm_std::{Deps, StdResult, Order, Uint128};
use cw_storage_plus::Bound;

use crate::msg::{ConfigResponse, NodeInfoResponse, ProofResponse, ProofsResponse, UserResponse, WhitelistedResponse, NodeReputationResponse};
use crate::state::{CONFIG, WHITELISTED_NODES, PROOFS, USERS, UNLOCKING_DEPOSITS, PROOF_BY_HASH}; // Added PROOF_BY_HASH
use crate::helpers::get_native_staked_amount;

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

/// Query contract config.
/// Returns the current configuration of the smart contract, including admin, version,
/// proof count, reputation threshold, and treasury address.
pub fn config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    
    Ok(ConfigResponse {
        admin: config.admin.to_string(),
        version: config.version,
        proof_count: config.proof_count,
        min_reputation_threshold: config.min_reputation_threshold,
        treasury: config.treasury.map(|addr| addr.to_string()),
        min_stake_tier1: config.min_stake_tier1,
        min_stake_tier2: config.min_stake_tier2,
        min_stake_tier3: config.min_stake_tier3,
        deposit_tier1: config.deposit_tier1,
        deposit_tier2: config.deposit_tier2,
        deposit_tier3: config.deposit_tier3,
        use_whitelist: config.use_whitelist,
        deposit_unlock_period_blocks: config.deposit_unlock_period_blocks,
    })
}

/// Query proof by ID.
/// Returns detailed information about a specific proof, identified by its unique ID.
pub fn proof(deps: Deps, id: u64) -> StdResult<ProofResponse> {
    let proof = PROOFS.load(deps.storage, id)?;
    
    Ok(ProofResponse {
        id: proof.id,
        data_hash: proof.data_hash,
        original_data_reference: proof.original_data_reference,
        data_owner: proof.data_owner, 
        metadata_json: proof.metadata_json,
        stored_at: proof.stored_at, // Renamed from verified_at
        stored_by: proof.stored_by.to_string(),
        tw_start: proof.tw_start, // Added
        tw_end: proof.tw_end,     // Added
        value_in: proof.value_in, // Added
        value_out: proof.value_out, // Added
        unit: proof.unit,         // Added
    })
}

/// Query proof by data hash.
/// Returns detailed information about a specific proof, identified by its data hash.
/// This is useful for verifying the existence and details of a proof when only the hash is known.
pub fn proof_by_hash(deps: Deps, data_hash: String) -> StdResult<ProofResponse> {
    let id = PROOF_BY_HASH.load(deps.storage, &data_hash)?;
    proof(deps, id)
}

/// Query all proofs with pagination.
/// Returns a list of proofs, allowing for pagination using `start_after` (proof ID) and `limit`.
/// Useful for iterating through all stored proofs.
pub fn proofs(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<ProofsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    
    let start = start_after.map(|id| Bound::exclusive(id));
    
    let proofs = PROOFS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            item.map(|(_, proof)| ProofResponse {
                id: proof.id,
                data_hash: proof.data_hash,
                original_data_reference: proof.original_data_reference,
                data_owner: proof.data_owner.clone(),
                metadata_json: proof.metadata_json,
                stored_at: proof.stored_at, // Renamed from verified_at
                stored_by: proof.stored_by.to_string(),
                tw_start: proof.tw_start, // Added
                tw_end: proof.tw_end,     // Added
                value_in: proof.value_in, // Added
                value_out: proof.value_out, // Added
                unit: proof.unit,         // Added
            })
        })
        .collect::<StdResult<Vec<_>>>()?;
    
    Ok(ProofsResponse { proofs })
}

/// Query user by address.
/// Returns information about a registered user, including their address, list of proof IDs they own,
/// and registration timestamp.
pub fn user(deps: Deps, address: String) -> StdResult<UserResponse> {
    let user = USERS.load(deps.storage, address)?;
    
    Ok(UserResponse {
        address: user.address.to_string(),
        proofs: user.proofs,
        registered_at: user.registered_at,
    })
}

/// Query proofs owned by a specific user with pagination.
/// Returns a list of proofs owned by the specified user, with support for pagination.
pub fn user_proofs(
    deps: Deps,
    address: String,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<ProofsResponse> {
    let user = USERS.load(deps.storage, address)?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    
    // Filter and paginate the proofs
    let start_pos = match start_after {
        Some(start) => user.proofs.iter().position(|&id| id > start).unwrap_or(user.proofs.len()),
        None => 0,
    };
    
    let proof_ids: Vec<u64> = user.proofs
        .iter()
        .skip(start_pos)
        .take(limit)
        .cloned()
        .collect();
    
    let mut proofs_resp: Vec<ProofResponse> = Vec::with_capacity(proof_ids.len());
    
    // Load each proof
    for id in proof_ids {
        let proof_from_storage = PROOFS.load(deps.storage, id)?;
        proofs_resp.push(ProofResponse {
            id: proof_from_storage.id,
            data_hash: proof_from_storage.data_hash,
            original_data_reference: proof_from_storage.original_data_reference,
            data_owner: proof_from_storage.data_owner.clone(),
            metadata_json: proof_from_storage.metadata_json,
            stored_at: proof_from_storage.stored_at, // Renamed from verified_at
            stored_by: proof_from_storage.stored_by.to_string(),
            tw_start: proof_from_storage.tw_start, // Added
            tw_end: proof_from_storage.tw_end,     // Added
            value_in: proof_from_storage.value_in, // Added
            value_out: proof_from_storage.value_out, // Added
            unit: proof_from_storage.unit,         // Added
        });
    }
    
    Ok(ProofsResponse { proofs: proofs_resp })
}

/// Query if an address is a whitelisted (or registered) node.
/// Returns true if the address is present in the `WHITELISTED_NODES` map, false otherwise.
/// Note: `WHITELISTED_NODES` now serves as the central registry for all active nodes.
pub fn is_whitelisted(deps: Deps, address: String) -> StdResult<WhitelistedResponse> {
    let is_whitelisted = WHITELISTED_NODES.has(deps.storage, address);
    
    Ok(WhitelistedResponse { is_whitelisted })
}

/// Query a node\'s reputation.
/// Returns the reputation score for a given node address.
/// If the node is not found in `WHITELISTED_NODES`, a default reputation of 0 is returned.
pub fn node_reputation(deps: Deps, address: String) -> StdResult<NodeReputationResponse> {
    // Check if node is whitelisted
    if !WHITELISTED_NODES.has(deps.storage, address.clone()) {
        return Ok(NodeReputationResponse {
            address,
            reputation: 0, // Default reputation for non-whitelisted nodes
        });
    }
    
    // Get node info
    let node = WHITELISTED_NODES.load(deps.storage, address.clone())?;
    
    Ok(NodeReputationResponse {
        address,
        reputation: node.reputation,
    })
}

/// Query comprehensive node information.
/// Returns detailed information about a node, including its reputation, and when it was added (registered).
/// Unlocking deposit information is also included if available.
pub fn node_info(deps: Deps, node_address: String) -> StdResult<NodeInfoResponse> {
    //let config = CONFIG.load(deps.storage)?;
    let validated_address = deps.api.addr_validate(&node_address)?;

    // Check for unlocking deposit information
    let unlocking_info = UNLOCKING_DEPOSITS.may_load(deps.storage, validated_address.to_string())?;
    let (unlocking_deposit_amount, unlocking_deposit_release_at_block) = match unlocking_info {
        Some(unlocking_deposit) => (Some(unlocking_deposit.amount), Some(unlocking_deposit.release_at_block)),
        None => (None, None),
    };

    match WHITELISTED_NODES.may_load(deps.storage, node_address.clone())? {
        Some(node) => {
            // Get native staked amount using the helper function
            let native_staked_amount = get_native_staked_amount(&deps.querier, &node.address)
                .unwrap_or_else(|_| Uint128::zero()); // Handle error case, e.g., by returning zero

            // Use the stored tier instead of recalculating it
            // The tier was determined at registration time based on stake requirements
            let current_tier = node.tier;

            Ok(NodeInfoResponse {
                address: node.address.to_string(),
                is_whitelisted: true, // Node is present in WHITELISTED_NODES
                reputation: node.reputation,
                added_at: Some(node.added_at),
                deposit: Some(node.deposit), // This is the active, locked deposit
                native_staked_amount: Some(native_staked_amount),
                tier: Some(current_tier), // Use the stored tier
                last_updated: Some(node.last_updated),
                proof_count: Some(node.proof_count),
                disputed_proofs: Some(node.disputed_proofs),
                unlocking_deposit_amount, // Added
                unlocking_deposit_release_at_block, // Added
            })
        }
        None => Ok(NodeInfoResponse {
            address: node_address,
            is_whitelisted: false, // Node not found, so not whitelisted/registered
            reputation: 0, // Default reputation for non-existent node
            added_at: None,
            deposit: None,
            native_staked_amount: None,
            tier: None,
            last_updated: None,
            proof_count: None,
            disputed_proofs: None,
            unlocking_deposit_amount, // Still include this, could be Some if node was removed but deposit is unlocking
            unlocking_deposit_release_at_block, // Same as above
        }),
    }
}

// TODO: Implement GetStakedAmount query as per HLD.
// This query would likely take a node address and return their natively staked C4E amount
// by querying the chain\'s staking module, similar to `get_native_staked_amount` in `execute.rs`.
// pub fn get_staked_amount(deps: Deps, node_address: String) -> StdResult<StakedAmountResponse> { ... }