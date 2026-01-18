use cosmwasm_std::{Deps, StdResult, Order, Uint128};
use cw_storage_plus::Bound;

use crate::msg::{ConfigResponse, NodeInfoResponse, ProofResponse, ProofsResponse, WhitelistedResponse, NodeReputationResponse};
use crate::state::{CONFIG, WHITELISTED_NODES, proofs, GATEWAY_PROOFS, UNLOCKING_DEPOSITS, PROOF_BY_HASH};
use crate::helpers::get_native_staked_amount;

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

/// Query contract config.
/// Returns the current configuration of the smart contract, including admin,
/// proof count, reputation threshold, treasury address, and DID contract address.
/// For contract version, use cw2::get_contract_version() query.
pub fn config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    
    Ok(ConfigResponse {
        admin: config.admin.to_string(),
        proof_count: config.proof_count,
        min_reputation_threshold: config.min_reputation_threshold,
        treasury: config.treasury.map(|addr| addr.to_string()),
        did_contract_address: config.did_contract_address.to_string(),
        min_stake_tier1: config.min_stake_tier1,
        min_stake_tier2: config.min_stake_tier2,
        min_stake_tier3: config.min_stake_tier3,
        deposit_tier1: config.deposit_tier1,
        deposit_tier2: config.deposit_tier2,
        deposit_tier3: config.deposit_tier3,
        use_whitelist: config.use_whitelist,
        deposit_unlock_period_blocks: config.deposit_unlock_period_blocks,
        max_batch_size: config.max_batch_size,
    })
}

/// Query proof by ID (Phase 1b).
/// Returns detailed information about a specific proof, identified by its unique ID.
pub fn proof(deps: Deps, id: u64) -> StdResult<ProofResponse> {
    let proof = proofs().load(deps.storage, id)?;
    
    Ok(ProofResponse {
        id: proof.id,
        worker_did: proof.worker_did,
        data_hash: proof.data_hash,
        tw_start: proof.tw_start,
        tw_end: proof.tw_end,
        batch_metadata: proof.batch_metadata,
        original_data_reference: proof.original_data_reference,
        metadata_json: proof.metadata_json,
        stored_at: proof.stored_at,
        stored_by: proof.stored_by.to_string(),
    })
}

/// Query proof by data hash.
/// Returns detailed information about a specific proof, identified by its data hash.
/// This is useful for verifying the existence and details of a proof when only the hash is known.
pub fn proof_by_hash(deps: Deps, data_hash: String) -> StdResult<ProofResponse> {
    let id = PROOF_BY_HASH.load(deps.storage, &data_hash)?;
    proof(deps, id)
}

/// Query all proofs with pagination (Phase 1b).
/// Returns a list of proofs, allowing for pagination using `start_after` (proof ID) and `limit`.
/// Useful for iterating through all stored proofs.
pub fn query_proofs(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<ProofsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    
    let start = start_after.map(|id| Bound::exclusive(id));
    
    let proofs_list = proofs()
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            item.map(|(_, proof)| ProofResponse {
                id: proof.id,
                worker_did: proof.worker_did,
                data_hash: proof.data_hash,
                batch_metadata: proof.batch_metadata,
                original_data_reference: proof.original_data_reference,
                metadata_json: proof.metadata_json,
                stored_at: proof.stored_at,
                stored_by: proof.stored_by.to_string(),
                tw_start: proof.tw_start,
                tw_end: proof.tw_end,
            })
        })
        .collect::<StdResult<Vec<_>>>()?;
    
    Ok(ProofsResponse { proofs: proofs_list })
}

/// Query proofs by worker DID with pagination (Phase 1b).
/// Uses secondary index for efficient worker_did lookups.
pub fn query_proofs_by_worker(
    deps: Deps,
    worker_did: String,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<ProofsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|id| Bound::exclusive(id));
    
    let proofs_list = proofs()
        .idx
        .worker
        .prefix(worker_did)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            item.map(|(_, proof)| ProofResponse {
                id: proof.id,
                worker_did: proof.worker_did,
                data_hash: proof.data_hash,
                batch_metadata: proof.batch_metadata,
                original_data_reference: proof.original_data_reference,
                metadata_json: proof.metadata_json,
                stored_at: proof.stored_at,
                stored_by: proof.stored_by.to_string(),
                tw_start: proof.tw_start,
                tw_end: proof.tw_end,
            })
        })
        .collect::<StdResult<Vec<_>>>()?;
    
    Ok(ProofsResponse { proofs: proofs_list })
}

/// Query proofs by gateway DID with pagination (Phase 1b).
/// Uses manual GATEWAY_PROOFS index for efficient gateway_did lookups.
pub fn query_proofs_by_gateway(
    deps: Deps,
    gateway_did: String,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<ProofsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|id| Bound::exclusive(id));
    
    let proof_ids: Vec<u64> = GATEWAY_PROOFS
        .prefix(&gateway_did)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(id, _)| id))
        .collect::<StdResult<Vec<_>>>()?;
    
    let mut proofs_list = Vec::with_capacity(proof_ids.len());
    for id in proof_ids {
        let proof = proofs().load(deps.storage, id)?;
        proofs_list.push(ProofResponse {
            id: proof.id,
            worker_did: proof.worker_did,
            data_hash: proof.data_hash,
            batch_metadata: proof.batch_metadata,
            original_data_reference: proof.original_data_reference,
            metadata_json: proof.metadata_json,
            stored_at: proof.stored_at,
            stored_by: proof.stored_by.to_string(),
            tw_start: proof.tw_start,
            tw_end: proof.tw_end,
        });
    }
    
    Ok(ProofsResponse { proofs: proofs_list })
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