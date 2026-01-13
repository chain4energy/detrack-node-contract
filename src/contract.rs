#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_json_binary};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::execute::{store_proof, update_admin, verify_proof, whitelist_node, remove_node, update_node_reputation, update_min_reputation_threshold, configure_treasury, register_node, add_deposit, unlock_deposit, claim_unlocked_deposit};
use crate::msg::{AdminExecuteMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, NodeExecuteMsg, QueryMsg};
use crate::query;
use crate::state::{Config, CONFIG};

// Contract name and version information
const CONTRACT_NAME: &str = "crates.io:detrack-node-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Handles contract instantiation.
/// Initializes the contract with admin, version, and other configurable parameters.
/// Sets up the initial state, including tier-based staking and deposit requirements,
/// the `use_whitelist` flag, and the deposit unlock period.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin = match msg.admin {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => info.sender,
    };

    let config = Config {
        admin,
        version: msg.version,
        proof_count: 0,
        min_reputation_threshold: 0, // Default minimum reputation threshold
        treasury: None, // Initialize treasury as None
        did_contract_address: deps.api.addr_validate(&msg.did_contract_address)?,
        // Initialize new config fields from InstantiateMsg
        min_stake_tier1: msg.min_stake_tier1,
        min_stake_tier2: msg.min_stake_tier2,
        min_stake_tier3: msg.min_stake_tier3,
        deposit_tier1: msg.deposit_tier1,
        deposit_tier2: msg.deposit_tier2,
        deposit_tier3: msg.deposit_tier3,
        use_whitelist: msg.use_whitelist,
        deposit_unlock_period_blocks: msg.deposit_unlock_period_blocks,
        max_batch_size: msg.max_batch_size,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", config.admin.to_string()) // Convert Addr to String for attribute
        .add_attribute("version", config.version)
        .add_attribute("deposit_unlock_period_blocks", msg.deposit_unlock_period_blocks.to_string()))
}

/// Handles contract execution.
/// Routes incoming `ExecuteMsg` to the appropriate handler function based on whether
/// it\'s an `AdminExecuteMsg` or a `NodeExecuteMsg`.
/// Admin messages are for administrative tasks like managing nodes and configuration.
/// Node messages are for core DeTrack operations like storing proofs and registering.
/// TODO: Add governance-related execute messages once HLD for governance is implemented.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Admin(admin_msg) => match admin_msg {
            AdminExecuteMsg::UpdateAdmin { new_admin } => update_admin(deps, info, new_admin),
            AdminExecuteMsg::WhitelistNode { node_address } => whitelist_node(deps, env, info, node_address),
            AdminExecuteMsg::RemoveNode { node_address } => remove_node(deps, info, node_address),
            AdminExecuteMsg::UpdateNodeReputation { node_address, reputation } => 
                update_node_reputation(deps, info, node_address, reputation),
            AdminExecuteMsg::UpdateMinReputationThreshold { threshold } =>
                update_min_reputation_threshold(deps, info, threshold),
            AdminExecuteMsg::ConfigureTreasury { treasury_address } =>
                configure_treasury(deps, info, treasury_address),
        },
        ExecuteMsg::Node(node_msg) => match node_msg {
            NodeExecuteMsg::StoreProof { 
                worker_did,
                data_hash, 
                tw_start,
                tw_end,
                batch_metadata,
                metadata_json,
            } => store_proof(
                deps, 
                env, 
                info, 
                worker_did,
                data_hash, 
                tw_start,
                tw_end,
                batch_metadata,
                metadata_json,
            ),
            NodeExecuteMsg::RegisterNode {} => register_node(deps, env, info),
            NodeExecuteMsg::AddDeposit {} => add_deposit(deps, env, info), // Added
            NodeExecuteMsg::VerifyProof { data_hash } => verify_proof(deps, env, info, data_hash),
            NodeExecuteMsg::UnlockDeposit {} => unlock_deposit(deps, env, info),
            NodeExecuteMsg::ClaimUnlockedDeposit {} => claim_unlocked_deposit(deps, env, info),
        },
    }
}

/// Handles contract queries.
/// Routes incoming `QueryMsg` to the appropriate query handler function.
/// Allows querying of contract state like configuration, proofs, user data, and node information.
/// TODO: Implement `GetStakedAmount` query as per HLD.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query::config(deps)?),
        QueryMsg::Proof { id } => to_json_binary(&query::proof(deps, id)?),
        QueryMsg::ProofByHash { data_hash } => to_json_binary(&query::proof_by_hash(deps, data_hash)?),
        QueryMsg::Proofs { start_after, limit } => to_json_binary(&query::query_proofs(deps, start_after, limit)?),
        QueryMsg::ProofsByWorker { worker_did, start_after, limit } => 
            to_json_binary(&query::query_proofs_by_worker(deps, worker_did, start_after, limit)?),
        QueryMsg::ProofsByGateway { gateway_did, start_after, limit } =>
            to_json_binary(&query::query_proofs_by_gateway(deps, gateway_did, start_after, limit)?),
        QueryMsg::IsWhitelisted { address } => to_json_binary(&query::is_whitelisted(deps, address)?),
        QueryMsg::NodeReputation { address } => to_json_binary(&query::node_reputation(deps, address)?),
        QueryMsg::NodeInfo { address } => to_json_binary(&query::node_info(deps, address)?),
    }
}

/// Handles contract migration.
/// Allows updating the contract to a new version. Currently, it only updates the
/// version string in the config. More complex migration logic can be added here if needed.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    msg: MigrateMsg,
) -> Result<Response, ContractError> {
    match msg {
        MigrateMsg::Migrate { new_version } => {
            // Migration logic
            let mut config = CONFIG.load(deps.storage)?;
            config.version = new_version.clone();
            CONFIG.save(deps.storage, &config)?;

            Ok(Response::new()
                .add_attribute("method", "migrate")
                .add_attribute("new_version", new_version))
        }
    }
}
