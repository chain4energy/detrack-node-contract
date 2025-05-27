# DeTrack Smart Contract Documentation

## 1. Overview

The DeTrack smart contract is a core component of the DeTrack system, designed to facilitate decentralized tracking and verification of data proofs. It manages node registration, data proof storage, user profiles, and a tiered staking/deposit mechanism for nodes. The contract ensures data integrity and incentivizes honest participation through reputation and staking.

Key functionalities include:
- Admin controls for managing the contract and node participants.
- Node registration with native token staking and contract-locked deposits, categorized into tiers.
- A mechanism for nodes to store data proofs (hashes of off-chain data).
- User registration and association with their submitted proofs.
- A deposit locking/unlocking mechanism with a configurable unbonding period.
- Optional whitelisting mode for node participation.
- // TODO: Implement slashing mechanism as per HLD.
- // TODO: Implement epoch-based reward system as per `DeTrack-node-reward-system.md`.

## 2. State Variables

The contract's state is managed by the following storage items and maps:

-   **`CONFIG: Item<Config>`**
    -   **Purpose:** Stores the global configuration of the contract.
    -   **Data Type:** `Config` struct.
    -   **Usage:** Accessed during instantiation and by various execute/query messages to retrieve operational parameters.
-   **`PROOFS: Map<u64, Proof>`**
    -   **Purpose:** Stores individual data proofs, keyed by a unique sequential ID.
    -   **Data Type:** `Proof` struct.
    -   **Usage:** Populated when nodes store proofs; queried to retrieve proof details.
-   **`PROOF_BY_HASH: Map<&str, u64>`**
    -   **Purpose:** Provides an index to look up a proof ID by its data hash.
    -   **Data Type:** `u64` (Proof ID).
    -   **Usage:** Used to quickly check for existing proofs and retrieve them by hash.
-   **`USERS: Map<&Addr, User>`**
    -   **Purpose:** Stores user profiles, keyed by their address.
    -   **Data Type:** `User` struct.
    -   **Usage:** Manages user registration and the list of proofs associated with each user.
-   **`NODES: Map<&Addr, Node>`**
    -   **Purpose:** Stores information about registered nodes, keyed by their address. This map serves as the primary registry for active nodes, whether they were whitelisted by an admin or registered via stake/deposit.
    -   **Data Type:** `Node` struct.
    -   **Usage:** Manages node registration details, reputation, active deposits, tier status, proof counts, and last update timestamps.
-   **`WHITELIST: Map<&Addr, bool>`**
    -   **Purpose:** If `use_whitelist` is true, this map can be used by an admin to explicitly mark addresses as whitelisted, potentially granting them permission to call `RegisterNode` or perform other actions. The primary `Node` data is stored in `NODES`.
    -   **Data Type:** `bool` (true if whitelisted).
    -   **Usage:** Checked by admin functions or potentially during node registration if `use_whitelist` is enabled and the logic requires explicit whitelisting before full registration.
-   **`UNLOCKING_DEPOSITS: Map<&Addr, UnlockingDeposit>`**
    -   **Purpose:** Stores information about node deposits that are currently in the unbonding/unlocking period.
    -   **Data Type:** `UnlockingDeposit` struct.
    -   **Usage:** Manages the lifecycle of deposits from the initiation of an unlock request until the deposit can be claimed by the node.

## 3. Key Structs

-   **`Config`**
    -   `admin: Addr`: The address of the contract administrator.
    -   `version: String`: The current version of the contract.
    -   `proof_count: u64`: A counter for the total number of proofs stored, used to assign unique IDs.
    -   `min_reputation_threshold: i32`: The minimum reputation a node must have to perform certain actions (e.g., store proofs).
    -   `treasury_address: Addr`: The address of the treasury contract/wallet where slashed funds or fees might be sent.
    -   `min_stake_tier1: Uint128`: Minimum native stake required for Tier 1.
    -   `min_stake_tier2: Uint128`: Minimum native stake required for Tier 2.
    -   `min_stake_tier3: Uint128`: Minimum native stake required for Tier 3.
    -   `deposit_tier1: Uint128`: Contract deposit required for Tier 1 (in the chain's native staking denom, e.g., "uc4e").
    -   `deposit_tier2: Uint128`: Contract deposit required for Tier 2.
    -   `deposit_tier3: Uint128`: Contract deposit required for Tier 3.
    -   `use_whitelist: bool`: A flag indicating whether nodes must be explicitly whitelisted by the admin to operate (if true) or if they can register directly via staking/deposit (if false).
    -   `deposit_unlock_period_blocks: u64`: The duration in blocks for which a node's deposit remains locked after initiating an unlock, before it can be claimed.

-   **`Proof`**
    -   `id: u64`: Unique identifier for the proof.
    -   `data_hash: String`: The hash of the off-chain data.
    -   `original_data_reference: Option<String>`: An optional reference (e.g., IPFS CID) to the original data.
    -   `data_owner: Option<Addr>`: Optional address of the entity that owns the data.
    -   `metadata_json: Option<String>`: Optional JSON string for additional metadata.
    -   `verified_at: Timestamp`: Timestamp of when the proof was stored.
    -   `stored_by: Addr`: Address of the node that stored the proof.

-   **`User`**
    -   `address: Addr`: The user's address.
    -   `proofs: Vec<u64>`: A list of IDs of proofs associated with this user.
    -   `registered_at: Timestamp`: Timestamp of when the user was registered.

-   **`Node`**
    -   `address: Addr`: The node's address.
    -   `reputation: i32`: The node's reputation score. Initialized to 0 upon registration or whitelisting.
    -   `added_at: Timestamp`: Timestamp of when the node was added/registered.
    -   `deposit: Uint128`: The amount of tokens (e.g., "uc4e") currently locked as an active deposit by the node in the contract.
    -   `tier: u8`: The operational tier of the node (1, 2, or 3), determined by their native stake during registration.
    -   `proof_count: u64`: Number of proofs successfully stored by this node.
    -   `disputed_proofs: u64`: Number of proofs from this node that have been disputed. // TODO: Implement dispute mechanism and link to slashing.
    -   `last_updated: Timestamp`: Timestamp of the last update to the node's record.

-   **`UnlockingDeposit`**
    -   `owner: Addr`: The address of the node whose deposit is unlocking.
    -   `amount: Coin`: The amount and denom of the deposit being unlocked.
    -   `release_at_block: u64`: The block height at which the deposit becomes claimable.

## 4. InstantiateMsg

Initializes the contract with global parameters.

-   **Purpose:** To set up the initial state of the DeTrack contract.
-   **Parameters:**
    -   `admin: Option<String>`: Optional address of the contract admin. If `None`, the instantiator becomes the admin.
    -   `version: String`: Contract version string.
    -   `min_stake_tier1: Uint128`: Minimum native stake for Tier 1.
    -   `min_stake_tier2: Uint128`: Minimum native stake for Tier 2.
    -   `min_stake_tier3: Uint128`: Minimum native stake for Tier 3.
    -   `deposit_tier1: Uint128`: Contract deposit amount for Tier 1 (in native_denom).
    -   `deposit_tier2: Uint128`: Contract deposit amount for Tier 2.
    -   `deposit_tier3: Uint128`: Contract deposit amount for Tier 3.
    -   `use_whitelist: bool`: If `true`, nodes may require admin whitelisting for certain actions or registration. If `false`, nodes primarily register via stake/deposit.
    -   `deposit_unlock_period_blocks: u64`: Number of blocks a deposit remains locked after initiating unlock.
    -   `min_reputation_threshold: i32`: Minimum reputation for nodes to perform actions like storing proofs.
    -   `treasury_address: String`: Address for the treasury.

## 5. Execute Messages (`ExecuteMsg`)

### 5.1. Admin Messages (`AdminExecuteMsg`)
Accessible only by the current contract admin.

-   **`UpdateAdmin { new_admin: String }`**
    -   **Purpose:** Changes the contract's admin address.
    -   **Parameters:** `new_admin` (String) - The address of the new admin.
    -   **Core Logic:** Validates `new_admin` and updates `Config.admin`.
    -   **Events:** `action: update_admin`, `old_admin`, `new_admin`.
    -   **Errors:** `ContractError::AdminOnly`.

-   **`WhitelistNode { node_address: String }`**
    -   **Purpose:** Explicitly whitelists a node address by the admin. This creates a basic `Node` entry with tier 0 (non-operational) if one doesn't exist or updates its `added_at` timestamp. Whitelisted nodes must still register with a deposit to become operational (tier 1+).
    -   **Parameters:** `node_address` (String) - The address of the node.
    -   **Core Logic:** If node not in `NODES`, creates a `Node` entry with default values (0 reputation, 0 deposit, 0 tier - non-operational). If node exists, updates `last_updated`. Sets `WHITELIST.save(node_addr, &true)`.
    -   **Events:** `action: whitelist_node`, `node_address`.
    -   **Errors:** `ContractError::AdminOnly`.

-   **`RemoveNode { node_address: String }`**
    -   **Purpose:** Removes a node's entry from the `NODES` map and `WHITELIST`.
    -   **Parameters:** `node_address` (String) - The address of the node.
    -   **Core Logic:** Removes the node from `NODES` and `WHITELIST`. Does not automatically handle their locked deposits; `UnlockDeposit` must be called by the node.
    -   **Events:** `action: remove_node`, `node_address`.
    -   **Errors:** `ContractError::AdminOnly`, `ContractError::NodeNotRegistered`.

-   **`UpdateNodeReputation { node_address: String, reputation: i32 }`**
    -   **Purpose:** Updates the reputation score of a registered node.
    -   **Parameters:** `node_address` (String), `reputation` (i32).
    -   **Core Logic:** Updates `Node.reputation` and `Node.last_updated`.
    -   **Events:** `action: update_node_reputation`, `node_address`, `new_reputation`.
    -   **Errors:** `ContractError::AdminOnly`, `ContractError::NodeNotRegistered`.

-   **`UpdateMinReputationThreshold { threshold: i32 }`**
    -   **Purpose:** Updates the global minimum reputation threshold.
    -   **Parameters:** `threshold` (i32).
    -   **Core Logic:** Updates `Config.min_reputation_threshold`.
    -   **Events:** `action: update_min_reputation_threshold`, `new_threshold`.
    -   **Errors:** `ContractError::AdminOnly`.

-   **`ConfigureTreasury { treasury_address: String }`**
    -   **Purpose:** Sets or updates the treasury address.
    -   **Parameters:** `treasury_address` (String).
    -   **Core Logic:** Updates `Config.treasury_address`.
    -   **Events:** `action: configure_treasury`, `treasury_address`.
    -   **Errors:** `ContractError::AdminOnly`.

-   **`SetWhitelistMode { use_whitelist: bool }`**
    -   **Purpose:** Enables or disables the whitelist mode.
    -   **Parameters:** `use_whitelist` (bool).
    -   **Core Logic:** Updates `Config.use_whitelist`.
    -   **Events:** `action: set_whitelist_mode`, `use_whitelist`.
    -   **Errors:** `ContractError::AdminOnly`.

-   **`UpdateTierParameters { tier1_min_stake: Option<Uint128>, tier1_deposit: Option<Uint128>, tier2_min_stake: Option<Uint128>, tier2_deposit: Option<Uint128>, tier3_min_stake: Option<Uint128>, tier3_deposit: Option<Uint128> }`**
    -   **Purpose:** Updates the stake and deposit requirements for one or more tiers.
    -   **Parameters:** Optional `Uint128` values for `min_stake_tierX` and `deposit_tierX` for tiers 1, 2, and 3.
    -   **Core Logic:** Updates the corresponding fields in `Config` if provided.
    -   **Events:** `action: update_tier_parameters`, includes changed parameters.
    -   **Errors:** `ContractError::AdminOnly`.
    -   **// TODO:** Clarify HLD: Does this affect existing nodes or only new registrations/tier changes?

-   **`SlashNode { node_address: String, amount: Uint128, reason: String }`**
    -   **Purpose:** Slashes a node's deposit.
    -   **Parameters:** `node_address` (String), `amount` (Uint128) to slash, `reason` (String).
    -   **Core Logic:** (Based on HLD, requires full implementation)
        -   Verify caller is admin.
        -   Load node from `NODES`.
        -   Check if node has sufficient deposit (active or unlocking).
        -   Reduce node's active deposit (`Node.deposit`) or unlocking deposit (`UNLOCKING_DEPOSITS.amount`).
        -   Transfer slashed `amount` to `Config.treasury_address`.
        -   Update node's reputation (`Node.reputation`, `Node.disputed_proofs`).
        -   Log event.
    -   **Events:** `action: slash_node`, `node_address`, `slashed_amount`, `remaining_deposit`, `reason`.
    -   **Errors:** `ContractError::AdminOnly`, `ContractError::NodeNotRegistered`, `ContractError::InsufficientDeposit` (not enough to slash), `ContractError::PaymentError` (if treasury transfer fails).
    -   **// TODO:** Full implementation based on HLD, including handling of slashing from unlocking deposits.

### 5.2. Node/User Messages (`NodeExecuteMsg`)

-   **`StoreProof { data_hash: String, original_data_reference: Option<String>, data_owner: Option<String>, metadata_json: Option<String> }`**
    -   **Purpose:** Allows a qualified node to store a new data proof.
    -   **Parameters:** As defined in struct. `data_owner` is `Option<String>` (address).
    -   **Access Control:**
        -   The caller (sender) must be a registered node (exist in `NODES`).
        -   The node's reputation must be >= `Config.min_reputation_threshold`.
        -   If `Config.use_whitelist` is `true`, the node must also be explicitly in the `WHITELIST` map (checked by `validate_node_permissions`).
    -   **Core Logic:**
        1.  Calls `validate_node_permissions` to check registration, reputation, and whitelist status if applicable.
        2.  Checks `data_hash` for emptiness and uniqueness (via `PROOF_BY_HASH`).
        3.  Increments `Config.proof_count`, creates `Proof` struct with current timestamp.
        4.  Saves proof to `PROOFS` (by ID) and `PROOF_BY_HASH` (by hash).
        5.  Updates `Node.proof_count` and `Node.last_updated`.
        6.  If `data_owner` is provided, validates the address. If valid, ensures user exists in `USERS` (registers if not) and adds proof ID to `User.proofs`.
    -   **Events:** `action: store_proof`, `proof_id`, `data_hash`, `stored_by`.
    -   **Errors:** `ContractError::NodeNotRegistered`, `ContractError::InsufficientNodeReputation`, `ContractError::NodeNotWhitelisted` (if applicable), `ContractError::InvalidInput` (empty hash), `ContractError::ProofAlreadyExists`, `ContractError::InvalidDataOwner`.

-   **`RegisterUser {}`**
    -   **Purpose:** Allows any address to register itself as a user in the system.
    -   **Parameters:** None.
    -   **Access Control:** Public.
    -   **Core Logic:** Checks if sender is already in `USERS`. If not, creates a new `User` entry with sender's address and current timestamp.
    -   **Events:** `action: register_user`, `user_address`.
    -   **Errors:** `ContractError::UserAlreadyRegistered`.

-   **`RegisterNode {}`**
    -   **Purpose:** Allows an address to register as a new node by verifying their native chain stake and submitting the required contract deposit. For whitelisted nodes (tier 0), this upgrades them to operational status.
    -   **Parameters:** None. Expects deposit in `info.funds` (must be native denom, e.g., "uc4e").
    -   **Access Control:** Public. Node cannot be already registered in `NODES` with tier > 0. If `Config.use_whitelist` is true, the node must also be present in the `WHITELIST` map or already whitelisted as tier 0.
    -   **Core Logic:**
        1.  Checks `NODES` to prevent re-registration of operational nodes (tier > 0).
        2.  If `Config.use_whitelist` is true, checks if sender is in `WHITELIST` or is already a tier 0 node.
        3.  Calls `get_native_staked_amount` to query sender's native stake on the chain. In test environments, returns a default stake amount (1000) when staking module queries fail.
        4.  Determines `tier` based on native stake against `Config.min_stake_tierX`. Fails if stake is insufficient for Tier 1.
        5.  Validates that `info.funds` contains a single coin of the native staking denom, matching `Config.deposit_tierX` for the determined tier.
        6.  Creates and saves a new `Node` entry with the address, initial reputation (0), `added_at` and `last_updated` timestamps, the locked `deposit` amount, and the determined `tier`.
    -   **Events:** `action: register_node`, `node_address`, `tier`, `deposit_amount`.
    -   **Errors:** `ContractError::NodeAlreadyRegistered`, `ContractError::NodeNotWhitelisted` (if `use_whitelist` is true and sender not in `WHITELIST`), `ContractError::InsufficientStake`, `ContractError::DepositDoesNotMatchTierRequirement`, `ContractError::StakingQueryError`, `ContractError::PaymentError` (invalid funds).

-   **`AddDeposit {}`**
    -   **Purpose:** Allows a registered node to add additional funds to their existing deposit without changing their tier.
    -   **Parameters:** None. Expects additional deposit in `info.funds` (must be native denom, e.g., "uc4e").
    -   **Access Control:** Caller must be a registered node in `NODES`.
    -   **Core Logic:**
        1.  Loads the calling node's `Node` entry from `NODES`.
        2.  Validates that `info.funds` contains a single coin of the native staking denom.
        3.  Adds the deposited amount to the node's existing `deposit`.
        4.  Updates `Node.last_updated` and saves the updated node entry.
    -   **Events:** `action: add_deposit`, `node_address`, `additional_amount`, `total_deposit`.
    -   **Errors:** `ContractError::NodeNotRegistered`, `ContractError::PaymentError` (invalid funds).

-   **`UnlockDeposit {}`**
    -   **Purpose:** Allows a registered node to initiate the unlocking process for their active contract deposit.
    -   **Parameters:** None.
    -   **Access Control:** Caller must be a registered node in `NODES`.
    -   **Core Logic:**
        1.  Loads the calling node's `Node` entry from `NODES`.
        2.  Checks `UNLOCKING_DEPOSITS` to prevent double unlock for the same node.
        3.  Ensures node has an active deposit (`node.deposit > 0`).
        4.  Creates an `UnlockingDeposit` entry with the node's current deposit (`node.deposit` and native denom from `querier.query_bonded_denom()`), owner (`info.sender`), and `release_at_block` (current block + `Config.deposit_unlock_period_blocks`).
        5.  Sets `node.deposit` to `Uint128::zero()` and updates `node.last_updated`. Saves updated `Node` entry.
        6.  Saves the new `UnlockingDeposit` entry.
    -   **Events:** `action: unlock_deposit`, `node_address`, `amount` (value and denom), `release_at_block`.
    -   **Errors:** `ContractError::NodeNotRegistered`, `ContractError::DepositAlreadyUnlocking`, `ContractError::NoDepositToUnlock`.

-   **`ClaimUnlockedDeposit {}`**
    -   **Purpose:** Allows a node to claim their deposit after the `deposit_unlock_period_blocks` has passed.
    -   **Parameters:** None.
    -   **Access Control:** Caller must have an entry in `UNLOCKING_DEPOSITS`.
    -   **Core Logic:**
        1.  Loads `UnlockingDeposit` for the sender from `UNLOCKING_DEPOSITS`.
        2.  Checks if `env.block.height >= release_at_block`.
        3.  If claimable, removes entry from `UNLOCKING_DEPOSITS`.
        4.  Sends the `amount` (Coin) back to the `owner` (sender) via `BankMsg::Send`.
    -   **Events:** `action: claim_unlocked_deposit`, `node_address`, `amount_claimed` (value and denom).
    -   **Errors:** `ContractError::NoUnlockedDepositToClaim`, `ContractError::DepositNotYetUnlocked`.

## 6. Query Messages (`QueryMsg`)

-   **`Config {}`**
    -   **Purpose:** Retrieves the current contract configuration.
    -   **Parameters:** None.
    -   **Response:** `ConfigResponse` (contains all fields from `Config` struct, addresses are `String`).

-   **`ProofById { proof_id: u64 }`**
    -   **Purpose:** Retrieves a specific proof by its ID.
    -   **Parameters:** `proof_id` (u64).
    -   **Response:** `ProofResponse` (contains all fields from `Proof` struct, addresses are `String`).
    -   **Errors:** `StdError::NotFound` if proof ID not found in `PROOFS`.

-   **`ProofByHash { data_hash: String }`**
    -   **Purpose:** Retrieves a specific proof by its data hash.
    -   **Parameters:** `data_hash` (String).
    -   **Response:** `ProofResponse`.
    -   **Errors:** `StdError::NotFound` if hash not in `PROOF_BY_HASH` or corresponding proof ID not in `PROOFS`.

-   **`UserProofs { user_address: String }`**
    -   **Purpose:** Retrieves all proof IDs associated with a user.
    -   **Parameters:** `user_address` (String).
    -   **Response:** `UserProofsResponse { proofs: Vec<u64> }`.
    -   **Errors:** `StdError::NotFound` if user not found in `USERS`.

-   **`NodeInfo { address: String }`**
    -   **Purpose:** Retrieves information about a specific registered node.
    -   **Parameters:** `address` (String) - The node's address.
    -   **Response:** `NodeInfoResponse` (contains fields from `Node` struct: `address`, `reputation`, `added_at`, `deposit`, `tier`, `proof_count`, `disputed_proofs`, `last_updated`; addresses are `String`).
    -   **Errors:** `StdError::NotFound` if node not found in `NODES`.

-   **`IsWhitelisted { address: String }`**
    -   **Purpose:** Checks if a given address is present in the `WHITELIST` map.
    -   **Parameters:** `address` (String).
    -   **Response:** `WhitelistedResponse { is_whitelisted: bool }`.

-   **`NodeReputation { address: String }`**
    -   **Purpose:** Retrieves the reputation of a specific node.
    -   **Parameters:** `address` (String).
    -   **Response:** `NodeReputationResponse { reputation: Option<i32> }`. (Returns `None` if node not found in `NODES`).

-   **`GetUnlockingDeposit { address: String }`**
    -   **Purpose:** Retrieves details of an unlocking deposit for a given address, if one exists.
    -   **Parameters:** `address` (String).
    -   **Response:** `Option<UnlockingDeposit>` (where `owner` is `String`, `amount` is `Coin`).

-   **`GetStakedAmount { address: String }`**
    -   **Purpose:** Retrieves the native staked amount for a given address by querying the chain's staking module.
    -   **Parameters:** `address` (String).
    -   **Response:** `StakedAmountResponse { amount: Uint128 }`.
    -   **// TODO:** Implement this query message in `query.rs`.

## 7. Custom Error Types (`ContractError`)

- `AdminOnlyOperation`: Operation can only be performed by the contract admin.
- `NodeNotWhitelisted`: Node is not in the `WHITELIST` (relevant when `use_whitelist` is true).
- `DepositAlreadyUnlocking`: The node already has a deposit in the unlocking process.
- `NoUnlockedDepositToClaim`: The node has no unlocked deposit waiting to be claimed.
- `Unauthorized { msg: String }`: General authorization failure.
- `InvalidInput { msg: String }`: General error for invalid input parameters.

## 8. Query Messages

- `Config {}`:
  - **Purpose:** Retrieves the contract's configuration.
  - **Response:** `ConfigResponse { admin: String, version: String, ... }`.

- `NodeInfo { address: String }`:
  - **Purpose:** Retrieves information about a specific node.
  - **Response:** `NodeInfoResponse { deposit: Option<Uint128>, ... }`.

- `ProofByHash { data_hash: String }`:
  - **Purpose:** Retrieves a proof by its data hash.
  - **Response:** `ProofResponse { data_hash: String, stored_by: String, ... }`.

- `IsWhitelisted { address: String }`:
  - **Purpose:** Checks if a given address is in the whitelist.
  - **Response:** `WhitelistedResponse { is_whitelisted: bool }`.

- `NodeReputation { address: String }`:
  - **Purpose:** Retrieves the reputation of a specific node.
  - **Response:** `NodeReputationResponse { reputation: Option<i32> }`.

## 9. Execute Messages

- `AdminExecuteMsg::WhitelistNode { node_address: String }`:
  - **Purpose:** Adds a node to the whitelist.

- `NodeExecuteMsg::RegisterNode {}`:
  - **Purpose:** Registers a node with the contract.

- `NodeExecuteMsg::StoreProof { data_hash: String, ... }`:
  - **Purpose:** Stores a proof in the contract.

- `NodeExecuteMsg::AddDeposit {}`:
  - **Purpose:** Adds a deposit for the node.

- `NodeExecuteMsg::UnlockDeposit {}`:
  - **Purpose:** Initiates the unlocking process for the node's deposit.

- `NodeExecuteMsg::ClaimUnlockedDeposit {}`:
  - **Purpose:** Claims the unlocked deposit for the node.

## 10. Tiered Node Registration

1. **Prerequisites (if `use_whitelist` is true):** The node's address must first be added to the `WHITELIST` by the admin via `AdminExecuteMsg::WhitelistNode`.
2. **Native Stake Check:** When `ExecuteMsg::Node(NodeExecuteMsg::RegisterNode {})` is called, the contract queries the chain's staking module to get the total amount of native tokens staked by the caller.
3. **Tier Determination:**
   - If staked amount >= `Config.min_stake_tier3`, node is Tier 3.
   - Else if staked amount >= `Config.min_stake_tier2`, node is Tier 2.
   - Else if staked amount >= `Config.min_stake_tier1`, node is Tier 1.
   - Otherwise, registration fails (`ContractError::InsufficientStake`).
4. **Deposit Requirement:** The caller must send funds with the `RegisterNode` message. These funds must match the deposit requirement for the determined tier.
