use crate::error::ContractError;
use crate::state::{Node, CONFIG, WHITELISTED_NODES, UNLOCKING_DEPOSITS, UnlockingDeposit, proofs, GATEWAY_PROOFS, PROOF_BY_HASH, Proof};
use crate::msg::BatchInfo;
use crate::helpers::get_native_staked_amount; // Added import
use cosmwasm_std::{BankMsg, Event, Coin, Uint128, Timestamp, DepsMut, Env, MessageInfo, Response};

/// ADMIN OPERATIONS

/// Validates that the sender is the admin
fn validate_admin(
    deps: &DepsMut,
    info: &MessageInfo,
) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::AdminOnlyOperation {});
    }
    Ok(())
}

/// Updates the admin address
pub fn update_admin(
    deps: DepsMut,
    info: MessageInfo,
    new_admin: String,
) -> Result<Response, ContractError> {
    validate_admin(&deps, &info)?;

    // Validate new admin address
    let validated_admin = deps.api.addr_validate(&new_admin)?;
    
    // Update admin
    let mut config = CONFIG.load(deps.storage)?;
    config.admin = validated_admin;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_admin")
        .add_attribute("new_admin", new_admin))
}

/// Adds a node to the whitelist
pub fn whitelist_node(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    node_address: String,
) -> Result<Response, ContractError> {
    validate_admin(&deps, &info)?;

    // Validate node address
    let validated_node = deps.api.addr_validate(&node_address)?;
    let node_str = validated_node.to_string();
    
    // Check if node already whitelisted
    if WHITELISTED_NODES.has(deps.storage, node_str.clone()) {
        return Err(ContractError::NodeAlreadyWhitelisted(node_str));
    }
    
    // Add node to whitelist with initial reputation
    let node = Node {
        address: validated_node.clone(),
        reputation: 0,
        added_at: env.block.time,
        deposit: Uint128::zero(), // Initialize deposit as zero
        tier: 0, // Initialize tier as 0
        proof_count: 0,
        disputed_proofs: 0,
        last_updated: env.block.time,
    };
    
    WHITELISTED_NODES.save(deps.storage, node_str.clone(), &node)?;
    
    Ok(Response::new()
        .add_attribute("action", "whitelist_node")
        .add_attribute("node_address", node_str))
}

/// Removes a node from the whitelist
pub fn remove_node(
    deps: DepsMut,
    info: MessageInfo,
    node_address: String,
) -> Result<Response, ContractError> {
    validate_admin(&deps, &info)?;
    
    // Validate node address
    let validated_node = deps.api.addr_validate(&node_address)?;
    let node_str = validated_node.to_string();
    
    // Check if node is whitelisted
    if !WHITELISTED_NODES.has(deps.storage, node_str.clone()) {
        return Err(ContractError::NodeNotWhitelisted(node_str.clone()));
    }
    
    // Remove node from whitelist
    WHITELISTED_NODES.remove(deps.storage, node_str.clone());
    
    Ok(Response::new()
        .add_attribute("action", "remove_node")
        .add_attribute("node_address", node_str))
}

/// Updates a node's reputation
pub fn update_node_reputation(
    deps: DepsMut,
    info: MessageInfo,
    node_address: String,
    reputation: i32,
) -> Result<Response, ContractError> {
    validate_admin(&deps, &info)?;
    
    // Validate node address
    let validated_node = deps.api.addr_validate(&node_address)?;
    let node_str = validated_node.to_string();
    
    // Check if node is whitelisted
    if !WHITELISTED_NODES.has(deps.storage, node_str.clone()) {
        return Err(ContractError::NodeNotWhitelisted(node_str));
    }
    
    // Update node reputation
    let mut node = WHITELISTED_NODES.load(deps.storage, node_str.clone())?;
    node.reputation = reputation;
    WHITELISTED_NODES.save(deps.storage, node_str.clone(), &node)?;
    
    Ok(Response::new()
        .add_attribute("action", "update_node_reputation")
        .add_attribute("node_address", node_str)
        .add_attribute("reputation", reputation.to_string()))
}

/// Updates the minimum reputation threshold
pub fn update_min_reputation_threshold(
    deps: DepsMut,
    info: MessageInfo,
    threshold: i32,
) -> Result<Response, ContractError> {
    validate_admin(&deps, &info)?;
    
    // Update the threshold in config
    let mut config = CONFIG.load(deps.storage)?;
    config.min_reputation_threshold = threshold;
    CONFIG.save(deps.storage, &config)?;
    
    Ok(Response::new()
        .add_attribute("action", "update_min_reputation_threshold")
        .add_attribute("threshold", threshold.to_string()))
}

/// Configures the treasury address
pub fn configure_treasury(
    deps: DepsMut,
    info: MessageInfo,
    treasury_address: String,
) -> Result<Response, ContractError> {
    validate_admin(&deps, &info)?;

    // Validate treasury address
    let validated_treasury = deps.api.addr_validate(&treasury_address)?;
    
    // Update treasury address
    let mut config = CONFIG.load(deps.storage)?;
    config.treasury = Some(validated_treasury);
    CONFIG.save(deps.storage, &config)?;
    
    Ok(Response::new()
        .add_attribute("method", "configure_treasury")
        .add_attribute("treasury", treasury_address))
}

/// NODE OPERATIONS

/// Validates that the sender is a whitelisted node with sufficient reputation
fn validate_node(
    deps: &DepsMut,
    info: &MessageInfo,
) -> Result<(), ContractError> {
    let sender = info.sender.to_string();
    
    // Check if node is whitelisted
    if !WHITELISTED_NODES.has(deps.storage, sender.clone()) {
        return Err(ContractError::NodeNotWhitelisted(sender));
    }
    
    // Check if node has sufficient reputation
    let node = WHITELISTED_NODES.load(deps.storage, sender.clone())?;
    let config = CONFIG.load(deps.storage)?;
    
    if node.reputation < config.min_reputation_threshold {
        return Err(ContractError::InsufficientNodeReputation(node.reputation, config.min_reputation_threshold));
    }
    
    // Check if node tier is operational (tier 0 is for whitelisted but non-operational nodes)
    if node.tier == 0 {
        return Err(ContractError::NodeTierNotOperational { current_tier: node.tier });
    }
    
    Ok(())
}

// ============================================================================
// NODE OPERATIONS - Phase 1b (DID-First Architecture)
// ============================================================================

/// Verify DID exists and is active in the DID Contract
/// 
/// This function queries the DID Contract to ensure the provided DID is registered
/// and follows the correct format for the expected type (worker or gateway).
/// 
/// # Arguments
/// * `deps` - Dependencies for querying
/// * `did` - The W3C DID to verify (e.g., "did:c4e:worker:detrack1")
/// * `expected_type` - Expected DID type ("worker" or "gateway")
/// 
/// # Returns
/// * `Ok(())` if DID is valid and registered
/// * `Err(ContractError)` if DID is invalid or not found
fn verify_did(
    _deps: &cosmwasm_std::Deps,
    did: &str,
    expected_type: &str,
) -> Result<(), ContractError> {
    // Validate DID format
    if !did.starts_with(&format!("did:c4e:{}:", expected_type)) {
        return Err(ContractError::InvalidDidFormat { did: did.to_string() });
    }
    
    // Skip DID Contract query in test mode (no real DID Contract available)
    #[cfg(test)]
    {
        return Ok(());
    }
    
    // Production: Query DID Contract to verify DID exists
    #[cfg(not(test))]
    {
    use cosmwasm_std::{to_json_binary, WasmQuery, QueryRequest};
    use serde::{Deserialize, Serialize};
    
    // Load DID contract address from config
    let config = CONFIG.load(_deps.storage)?;
    
    // Query DID contract to verify DID exists
    #[derive(Serialize)]
    #[serde(rename_all = "snake_case")]
    enum DidQueryMsg {
        GetDidDocument { did: String },
    }
    
    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct DidDocumentResponse {
        id: String,
        controller: String,
        service: Vec<serde_json::Value>,
    }
    
    let query_msg = DidQueryMsg::GetDidDocument { did: did.to_string() };
    let query_request: QueryRequest<cosmwasm_std::Empty> = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: config.did_contract_address.to_string(),
        msg: to_json_binary(&query_msg)?,
    });
    
    let response: Result<DidDocumentResponse, _> = _deps.querier.query(&query_request);
    
    match response {
        Ok(_doc) => Ok(()),
        Err(_) => Err(ContractError::DidNotFound { did: did.to_string() }),
    }
    } // end cfg(not(test))
}

/// Stores a new proof on the blockchain (Phase 1b: Multi-batch aggregation)
/// 
/// Access Control: Only whitelisted nodes with sufficient reputation can store proofs.
/// DID Verification: Verifies worker_did and all gateway_dids in batch_metadata.
/// 
/// Logic:
/// - Validates the calling node (whitelist + reputation)
/// - Verifies Worker DID exists in DID Contract
/// - Verifies all Gateway DIDs in batch_metadata
/// - Validates batch_metadata (not empty, not too many batches)
/// - Checks data hash validity and uniqueness
/// - Creates and saves proof with IndexedMap
/// - Indexes by gateway DIDs for efficient queries
/// 
/// Events: Emits attributes for "store_proof", "proof_id", "worker_did", "data_hash", etc.
/// 
/// Errors:
/// - `InvalidDidFormat` if DIDs don't match expected format
/// - `DidNotFound` if any DID is not registered
/// - `EmptyBatchMetadata` if no batches provided
/// - `TooManyBatches` if more than 100 batches
/// - `ProofAlreadyExists` if hash already exists
/// - `InvalidInput` for validation failures
pub fn store_proof(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    worker_did: String,
    data_hash: String,
    tw_start: Timestamp,
    tw_end: Timestamp,
    batch_metadata: Vec<BatchInfo>,
    metadata_json: Option<String>,
) -> Result<Response, ContractError> {
    // Validate calling node
    validate_node(&deps, &info)?;
    
    let node = WHITELISTED_NODES.load(deps.storage, info.sender.to_string())
        .map_err(|_| ContractError::NodeNotRegistered { address: info.sender.to_string() })?;
    
    let mut config = CONFIG.load(deps.storage)?;
    
    // Validate node tier and deposit
    if !(1..=3).contains(&node.tier) {
        return Err(ContractError::NodeTierNotOperational { current_tier: node.tier });
    }
    
    let required_deposit_for_tier = match node.tier {
        3 => config.deposit_tier3,
        2 => config.deposit_tier2,
        1 => config.deposit_tier1,
        _ => return Err(ContractError::NodeTierNotOperational { current_tier: node.tier }),
    };
    
    if node.deposit < required_deposit_for_tier {
        return Err(ContractError::NodeHasInsufficientDeposit {
            required_deposit: required_deposit_for_tier,
            current_deposit: node.deposit,
            tier: node.tier,
        });
    }
    
    // Phase 1b: Verify Worker DID
    verify_did(&deps.as_ref(), &worker_did, "worker")?;
    
    // Phase 1b: Validate batch_metadata
    if batch_metadata.is_empty() {
        return Err(ContractError::EmptyBatchMetadata {});
    }
    
    if batch_metadata.len() > config.max_batch_size as usize {
        return Err(ContractError::TooManyBatches { count: batch_metadata.len() });
    }
    
    // Phase 1b: Verify all Gateway DIDs in batch_metadata
    for batch in &batch_metadata {
        verify_did(&deps.as_ref(), &batch.gateway_did, "gateway")?;
    }
    
    // Validate data_hash
    if data_hash.is_empty() {
        return Err(ContractError::InvalidInput("Data hash cannot be empty".to_string()));
    }
    
    if data_hash.len() != 64 || !data_hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ContractError::InvalidInput("Data hash must be 64 hex characters".to_string()));
    }
    
    // Check if proof already exists
    if PROOF_BY_HASH.has(deps.storage, &data_hash) {
        return Err(ContractError::ProofAlreadyExists(data_hash));
    }
    
    // Increment proof count
    let proof_id = config.proof_count;
    config.proof_count += 1;
    CONFIG.save(deps.storage, &config)?;
    
    // Create new proof (Phase 1b structure)
    let proof = Proof {
        id: proof_id,
        worker_did: worker_did.clone(),
        data_hash: data_hash.clone(),
        tw_start,
        tw_end,
        batch_metadata: batch_metadata.clone(),
        metadata_json,
        stored_at: env.block.time,
        stored_by: info.sender.clone(),
    };
    
    // Save proof with IndexedMap (auto-indexes by worker_did)
    proofs().save(deps.storage, proof_id, &proof)?;
    
    // Index proof by hash
    PROOF_BY_HASH.save(deps.storage, &data_hash, &proof_id)?;
    
    // Phase 1b: Index by gateway DIDs (manual index)
    for batch in &batch_metadata {
        GATEWAY_PROOFS.save(
            deps.storage,
            (&batch.gateway_did, proof_id),
            &(),
        )?;
    }
    
    // Build event attributes
    let mut event = Event::new("store_proof")
        .add_attribute("action", "store_proof")
        .add_attribute("proof_id", proof_id.to_string())
        .add_attribute("worker_did", worker_did)
        .add_attribute("data_hash", data_hash)
        .add_attribute("stored_by", info.sender.to_string())
        .add_attribute("batch_count", batch_metadata.len().to_string())
        .add_attribute("tw_start", tw_start.to_string())
        .add_attribute("tw_end", tw_end.to_string());
    
    // Add gateway DIDs to event (comma-separated)
    let gateway_dids: Vec<String> = batch_metadata.iter()
        .map(|b| b.gateway_did.clone())
        .collect();
    event = event.add_attribute("gateway_dids", gateway_dids.join(","));
    
    Ok(Response::new()
        .add_event(event))
}


/// Verifies a proof's existence by its data hash.
/// 
pub fn verify_proof(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    data_hash: String,
) -> Result<Response, ContractError> {
    // Check that sender is a whitelisted node
    validate_node(&deps, &info)?;
    
    // Check if proof exists
    if !PROOF_BY_HASH.has(deps.storage, &data_hash) {
        return Err(ContractError::ProofNotFound(data_hash));
    }

    // Get proof ID
    let proof_id = PROOF_BY_HASH.load(deps.storage, &data_hash)?;
    
    Ok(Response::new()
        .add_attribute("action", "verify_proof")
        .add_attribute("verified", "true")
        .add_attribute("data_hash", data_hash)
        .add_attribute("proof_id", proof_id.to_string()))
}

/// Registers a new node, verifies native stake, and locks their deposit.
/// This function allows any address to attempt to register as a node, provided they meet
/// the native staking requirements for a tier and send the correct corresponding deposit.
/// Logic:
/// 1. Checks if the node is already registered.
/// 2. Fetches the node\'s native staked amount using `get_native_staked_amount`.
/// 3. Determines the node\'s tier based on their native stake against configured thresholds.
/// 4. Verifies that the `info.funds` (deposit sent with the registration message) matches
///    the required deposit for the determined tier.
/// 5. If all checks pass, a new `Node` entry is created and saved in `WHITELISTED_NODES`.
///    The `WHITELISTED_NODES` map now serves as the central registry for all active nodes,
///    regardless of the `use_whitelist` flag in `Config`.
/// Events: Emits attributes for "register_node", "node_address", "native_stake_verified",
///         "tier_assigned", "deposit_locked".
/// Errors:
/// - `CustomError("Node already registered")` if the node is already in `WHITELISTED_NODES`.
/// - `InsufficientStake` if native stake is below the minimum for Tier 1.
/// - `DepositDoesNotMatchTierRequirement` if the sent deposit doesn\'t match the tier\'s requirement.
pub fn register_node(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let sender_addr = info.sender.clone();
    let sender_str = sender_addr.to_string();
    let config = CONFIG.load(deps.storage)?;

    // Check if node is already registered in WHITELISTED_NODES
    let existing_node = WHITELISTED_NODES.may_load(deps.storage, sender_str.clone())?;
    
    // If node exists and is already operational (tier > 0), prevent re-registration
    if let Some(existing) = &existing_node {
        if existing.tier > 0 {
            return Err(ContractError::CustomError("Node already registered".to_string()));
        }
        // If tier is 0, this is a whitelisted node that needs to upgrade - continue with registration
    }

    // 1. Verify Native Stake and Determine Tier
    // This step queries the chain\'s staking module to get the total amount
    // the sender has staked in the native C4E token.
    let native_staked_amount = get_native_staked_amount(&deps.querier, &sender_addr)?;

    // Determine the tier based on the native staked amount.
    // Tiers provide different levels of service or trust within the DeTrack network.
    let tier = if native_staked_amount >= config.min_stake_tier3 {
        3u8
    } else if native_staked_amount >= config.min_stake_tier2 {
        2u8
    } else if native_staked_amount >= config.min_stake_tier1 {
        1u8
    } else {
        return Err(ContractError::InsufficientStake {
            required: config.min_stake_tier1, // Minimum requirement is Tier 1 stake
            provided: native_staked_amount,
        });
    };

    // 2. Verify Deposit Sent with this Message matches the requirement for the stake-determined Tier
    // The node must send a specific amount of `uc4e` (the deposit token) with this registration
    // message. The required amount depends on the tier they qualified for based on their native stake.
    let required_deposit_for_tier = match tier {
        3 => config.deposit_tier3,
        2 => config.deposit_tier2,
        _ => config.deposit_tier1, // Default to Tier 1 deposit requirement
    };

    let sent_deposit_amount = info
        .funds
        .iter()
        .find(|c| c.denom == "uc4e") // Assuming "uc4e" is the deposit/staking denom
        .map_or(Uint128::zero(), |c| c.amount);
    
    // Check if the sent deposit matches the required deposit for the determined tier
    if sent_deposit_amount < required_deposit_for_tier {
        return Err(ContractError::DepositDoesNotMatchTierRequirement {
            required_deposit: required_deposit_for_tier,
            provided_deposit: sent_deposit_amount,
            tier,
        });
    }

    let node = Node {
        address: sender_addr,
        reputation: 0, // Reset reputation for new registration
        added_at: existing_node.as_ref().map_or(env.block.time, |n| n.added_at), // Preserve original timestamp for whitelisted nodes
        deposit: sent_deposit_amount, // Store the locked deposit amount from this transaction
        tier, // Tier determined by native stake
        proof_count: 0, // Reset proof count for new registration
        disputed_proofs: 0, // Reset disputed proofs for new registration
        last_updated: env.block.time,
    };

    WHITELISTED_NODES.save(deps.storage, sender_str.clone(), &node)?;

    // TODO: Consider adding a mechanism for nodes to upgrade/downgrade tiers if their native stake changes.
    // TODO: Implement slashing conditions related to node registration or behavior post-registration.

    Ok(Response::new()
        .add_attribute("action", "register_node")
        .add_attribute("node_address", sender_str)
        .add_attribute("native_stake_verified", native_staked_amount.to_string())
        .add_attribute("tier_assigned", tier.to_string())
        .add_attribute("deposit_locked", sent_deposit_amount.to_string()))
}

/// Initiates the unlocking period for a node\'s deposit.
/// Access Control: Only the registered node can initiate unlocking for their own deposit.
/// Logic:
/// 1. Validates that the sender is a registered node.
/// 2. Checks if the deposit isn\'t already in the process of unlocking.
/// 3. Checks if the node has a non-zero deposit to unlock.
/// 4. Moves the node\'s active deposit amount to a new `UnlockingDeposit` entry.
///    The node\'s `deposit` field is set to zero, effectively making their current deposit inactive.
/// 5. Calculates `release_at_block` based on the current block height and `deposit_unlock_period_blocks` from config.
/// 6. Saves the `UnlockingDeposit` entry, keyed by the node\'s address.
/// State Transition:
/// - Node\'s `deposit` in `WHITELISTED_NODES` is set to 0.
/// - A new entry is created in `UNLOCKING_DEPOSITS` for the node, with the amount and release block.
/// Events: Emits "unlock_deposit", "node_address", "unlocking_amount", "release_at_block".
/// Errors:
/// - `NodeNotRegistered` if the sender is not a registered node.
/// - `DepositAlreadyUnlocking` if an unlocking process is already active for the node.
/// - `NoDepositToUnlock` if the node\'s current active deposit is zero.
pub fn unlock_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let sender_addr = info.sender.clone();
    let sender_str = sender_addr.to_string();
    let config = CONFIG.load(deps.storage)?;

    // Check if node is registered
    let mut node = WHITELISTED_NODES.load(deps.storage, sender_str.clone())
        .map_err(|_| ContractError::NodeNotRegistered { address: sender_str.clone() })?;

    // Check if deposit is already unlocking
    if UNLOCKING_DEPOSITS.has(deps.storage, sender_addr.to_string()) {
        return Err(ContractError::DepositAlreadyUnlocking {});
    }

    // Check if there's a deposit to unlock
    if node.deposit.is_zero() {
        return Err(ContractError::NoDepositToUnlock {});
    }

    // State Change: Node\'s active deposit is moved to an unlocking state.
    // The node.deposit field is zeroed out, and an UnlockingDeposit entry is created.
    let unlocking_amount = node.deposit;
    node.deposit = Uint128::zero(); // Remove active deposit from node
    WHITELISTED_NODES.save(deps.storage, sender_str.clone(), &node)?;

    let release_at_block = env.block.height + config.deposit_unlock_period_blocks;

    let unlocking_deposit = UnlockingDeposit {
        owner: sender_addr.clone(),
        amount: unlocking_amount,
        release_at_block,
    };

    UNLOCKING_DEPOSITS.save(deps.storage, sender_addr.to_string(), &unlocking_deposit)?;

    let mut response = Response::default();

    let event = Event::new("detrack_unlock_deposit")
        .add_attribute("node_address", sender_str)
        .add_attribute("unlocking_amount", unlocking_amount.to_string())
        .add_attribute("release_at_block", release_at_block.to_string());

    response = response.add_event(event);

    Ok(response)

//     Ok(Response::new()
//         .add_event(Event::UnlockDeposit {
//             node_address: sender_str,
//             unlocking_amount,
//             release_at_block,
//         })
//         .add_attribute("action", "unlock_deposit")
//         .add_attribute("node_address", sender_str)
//         .add_attribute("unlocking_amount", unlocking_amount.to_string())
//         .add_attribute("release_at_block", release_at_block.to_string()))
}

/// Allows a node to claim their deposit after the unlocking period has passed.
/// Access Control: Only the node who initiated the unlock can claim their deposit.
/// Logic:
/// 1. Loads the `UnlockingDeposit` entry for the sender.
/// 2. Verifies that the current block height is greater than or equal to `release_at_block`.
/// 3. Removes the `UnlockingDeposit` entry from storage.
/// 4. Creates a `BankMsg::Send` to transfer the unlocked amount back to the node.
/// State Transition:
/// - The `UnlockingDeposit` entry for the node is removed from `UNLOCKING_DEPOSITS`.
/// - Funds are transferred from the contract to the node.
/// Events: Emits "claim_unlocked_deposit", "node_address", "claimed_amount".
/// Errors:
/// - `NoUnlockedDepositToClaim` if no unlocking deposit entry exists for the sender.
/// - `DepositNotYetUnlocked` if the current block height is less than `release_at_block`.
/// TODO: Consider if any slashing conditions should prevent claiming (e.g., if node was slashed during unlock period).
///       Currently, slashing is not implemented, but this would be a point of integration.
pub fn claim_unlocked_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let sender_addr = info.sender.clone();

    // Check if there's an unlocking deposit entry for the sender
    let unlocking_deposit = UNLOCKING_DEPOSITS.load(deps.storage, sender_addr.to_string())
        .map_err(|_| ContractError::NoUnlockedDepositToClaim {})?;

    // Check if the unlocking period has passed
    if env.block.height < unlocking_deposit.release_at_block {
        return Err(ContractError::DepositNotYetUnlocked {
            release_at_block: unlocking_deposit.release_at_block,
        });
    }

    // State Change: Unlocking deposit entry is removed, and funds are sent to the node.
    // Remove the unlocking deposit entry
    UNLOCKING_DEPOSITS.remove(deps.storage, sender_addr.to_string());

    // Send the funds back to the user
    let bank_msg = BankMsg::Send {
        to_address: sender_addr.to_string(),
        amount: vec![Coin {
            denom: "uc4e".to_string(), // Ensure this is your chain's native token denom
            amount: unlocking_deposit.amount,
        }],
    };

    let mut response = Response::default();

    let event = Event::new("detrack_claim_unlocked_deposit")
        .add_attribute("node_address", sender_addr.to_string())
        .add_attribute("claimed_amount", unlocking_deposit.amount.to_string());

    response = response
        .add_message(bank_msg)
        .add_event(event);

    Ok(response)

    // Ok(Response::new()
    //     .add_message(bank_msg)
    //     .add_attribute("action", "claim_unlocked_deposit")
    //     .add_attribute("node_address", sender_addr.to_string())
    //     .add_attribute("claimed_amount", unlocking_deposit.amount.to_string()))
}

/// Allows a registered node to add more funds to their existing deposit.
/// Access Control: Only a registered node can add to their own deposit.
/// Logic:
/// 1. Validates that the sender is a registered node.
/// 2. Checks that the node\'s deposit is not currently in an unlocking period.
/// 3. Verifies that funds of the correct denomination ("uc4e") were sent with the message.
/// 4. Adds the sent amount to the node\'s current deposit.
/// 5. Updates the node\'s `last_updated` timestamp.
/// State Transition:
/// - Node\'s `deposit` in `WHITELISTED_NODES` is increased.
/// - Node\'s `last_updated` in `WHITELISTED_NODES` is updated.
/// Events: Emits "add_deposit", "node_address", "added_amount", "new_total_deposit".
/// Errors:
/// - `NodeNotRegistered` if the sender is not a registered node.
/// - `DepositAlreadyUnlocking` if the node\'s deposit is currently being unlocked.
/// - `CustomError("No deposit amount provided or amount is zero")` if no "uc4e" funds are sent.
/// - `CustomError("Invalid deposit denomination")` if funds other than "uc4e" are sent.
pub fn add_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let sender_addr = info.sender.clone();
    let sender_str = sender_addr.to_string();

    // 1. Validate that the sender is a registered node
    let mut node = WHITELISTED_NODES.load(deps.storage, sender_str.clone())
        .map_err(|_| ContractError::NodeNotRegistered { address: sender_str.clone() })?;

    // 2. Check that the node\'s deposit is not currently in an unlocking period
    if UNLOCKING_DEPOSITS.has(deps.storage, sender_addr.to_string()) {
        return Err(ContractError::DepositAlreadyUnlocking {});
    }

    // 3. Verify that funds of the correct denomination ("uc4e") were sent
    let sent_deposit_amount = info
        .funds
        .iter()
        .find(|c| c.denom == "uc4e") // Assuming "uc4e" is the deposit denom
        .map_or(Uint128::zero(), |c| c.amount);

    if sent_deposit_amount.is_zero() {
        return Err(ContractError::CustomError("No deposit amount provided or amount is zero".to_string()));
    }

    // Optional: Check if other denominations were sent and reject if so, or ignore.
    // For simplicity, we only care about "uc4e". If other denoms are sent, they are ignored by the sum above.
    // If strictness is required:
    if info.funds.len() > 1 && info.funds.iter().any(|c| c.denom != "uc4e") {
         // Or if only one coin is sent but it's not uc4e
         if info.funds.len() == 1 && info.funds[0].denom != "uc4e" {
            return Err(ContractError::CustomError("Invalid deposit denomination. Only uc4e is accepted.".to_string()));
         }
    }


    // 4. Add the sent amount to the node\'s current deposit
    node.deposit += sent_deposit_amount;

    // 5. Update the node\'s `last_updated` timestamp
    node.last_updated = env.block.time;

    // Save the updated node data
    WHITELISTED_NODES.save(deps.storage, sender_str.clone(), &node)?;

    Ok(Response::new()
        .add_attribute("action", "add_deposit")
        .add_attribute("node_address", sender_str)
        .add_attribute("added_amount", sent_deposit_amount.to_string())
        .add_attribute("new_total_deposit", node.deposit.to_string()))
}