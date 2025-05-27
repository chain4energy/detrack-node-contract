use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Admin only operation")]
    AdminOnlyOperation {},

    #[error("Node not whitelisted: {0}")]
    NodeNotWhitelisted(String),

    #[error("Node already whitelisted: {0}")]
    NodeAlreadyWhitelisted(String),

    #[error("Insufficient node reputation: {0} (required: {1})")]
    InsufficientNodeReputation(i32, i32),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Proof already exists: {0}")]
    ProofAlreadyExists(String),
    
    #[error("Proof not found: {0}")]
    ProofNotFound(String),
    
    #[error("User not found: {0}")]
    UserNotFound(String),
    
    #[error("Invalid data owner: {0}")]
    InvalidDataOwner(String),
    
    #[error("Invalid data hash: {0}")]
    InvalidDataHash(String),
    
    #[error("Custom error: {0}")]
    CustomError(String),

    #[error("Deposit already unlocking")]
    DepositAlreadyUnlocking {},

    #[error("No deposit to unlock")]
    NoDepositToUnlock {},

    #[error("Deposit not yet unlocked. Will be released at block {release_at_block}")]
    DepositNotYetUnlocked { release_at_block: u64 },

    #[error("No unlocked deposit to claim")]
    NoUnlockedDepositToClaim {},

    #[error("Insufficient stake. Required: {required}, provided: {provided}")]
    InsufficientStake { required: Uint128, provided: Uint128 },

    #[error("Deposit does not match tier requirement. Required: {required_deposit}, provided: {provided_deposit}, for tier: {tier}")]
    DepositDoesNotMatchTierRequirement { required_deposit: Uint128, provided_deposit: Uint128, tier: u8 },

    #[error("Staking query error: {error}")]
    StakingQueryError { error: String },

    #[error("Node {address} not registered")]
    NodeNotRegistered { address: String },

    #[error("Node tier {current_tier} is not operational")]
    NodeTierNotOperational { current_tier: u8 },

    #[error("Node has insufficient deposit. Current: {current_deposit}, Required: {required_deposit} for tier {tier}")]
    NodeHasInsufficientDeposit { current_deposit: Uint128, required_deposit: Uint128, tier: u8 },
}