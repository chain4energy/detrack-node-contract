#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, coins, Empty, Uint128, Timestamp};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    use crate::contract::{execute, instantiate, query};
    use crate::msg::{
        ExecuteMsg, InstantiateMsg, QueryMsg, ConfigResponse, ProofResponse, NodeExecuteMsg,
        AdminExecuteMsg, NodeInfoResponse, WhitelistedResponse, NodeReputationResponse,
    };
    use crate::error::ContractError;

    const ADMIN: &str = "admin";
    const USER: &str = "user";
    const USER2: &str = "user2";
    const NODE_USER: &str = "node1";
    const DATA_HASH: &str = "532eaabd9574880dbf76b9b8cc00832c20a6ec113d682299550d7a6e0f345e25";
    const NATIVE_DENOM: &str = "uc4e";

    // Helper functions
    fn detrack_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    fn default_instantiate_msg() -> InstantiateMsg {
        InstantiateMsg {
            admin: Some(ADMIN.to_string()),
            version: "1.0.0".to_string(),
            min_stake_tier1: Uint128::new(1000),
            min_stake_tier2: Uint128::new(5000),
            min_stake_tier3: Uint128::new(10000),
            deposit_tier1: Uint128::new(100), // uc4e
            deposit_tier2: Uint128::new(500), // uc4e
            deposit_tier3: Uint128::new(1000), // uc4e
            use_whitelist: true,
            deposit_unlock_period_blocks: 100,
        }
    }

    fn mock_app() -> App {
        App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked(ADMIN), coins(1_000_000, NATIVE_DENOM))
                .unwrap();
            router
                .bank
                .init_balance(storage, &Addr::unchecked(USER), coins(1_000_000, NATIVE_DENOM))
                .unwrap();
            router
                .bank
                .init_balance(storage, &Addr::unchecked(USER2), coins(1_000_000, NATIVE_DENOM))
                .unwrap();
            router
                .bank
                .init_balance(storage, &Addr::unchecked(NODE_USER), coins(1_000_000, NATIVE_DENOM))
                .unwrap();
        })
    }

    #[test]
    fn proper_instantiation() {
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());
        let msg = default_instantiate_msg();

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "DeTrack Test",
                Some(ADMIN.to_string()),
            )
            .unwrap();

        // Query the config
        let config_response: ConfigResponse = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Config {})
            .unwrap();

        assert_eq!(config_response.admin, Addr::unchecked(ADMIN));
        assert_eq!(config_response.version, msg.version);
        assert_eq!(config_response.min_stake_tier1, msg.min_stake_tier1);
        assert_eq!(config_response.min_stake_tier2, msg.min_stake_tier2);
        assert_eq!(config_response.min_stake_tier3, msg.min_stake_tier3);
        assert_eq!(config_response.deposit_tier1, msg.deposit_tier1);
        assert_eq!(config_response.deposit_tier2, msg.deposit_tier2);
        assert_eq!(config_response.deposit_tier3, msg.deposit_tier3);
        assert_eq!(config_response.use_whitelist, msg.use_whitelist);
        assert_eq!(
            config_response.deposit_unlock_period_blocks,
            msg.deposit_unlock_period_blocks
        );
    }

    #[test]
    fn test_store_proof() {
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());

        // Instantiate with use_whitelist = true (default from helper)
        let instantiate_msg = default_instantiate_msg();
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked(ADMIN),
                &instantiate_msg,
                &[],
                "DeTrack",
                None,
            )
            .unwrap();

        // Whitelist the USER as a node first (since use_whitelist is true)
        let whitelist_msg = ExecuteMsg::Admin(AdminExecuteMsg::WhitelistNode {
            node_address: USER.to_string(),
        });

        app.execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &whitelist_msg,
            &[],
        )
        .unwrap();

        // USER needs to register as a node to become operational (tier 1+)
        let register_msg = ExecuteMsg::Node(NodeExecuteMsg::RegisterNode {});
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &register_msg,
            &coins(instantiate_msg.deposit_tier1.u128(), NATIVE_DENOM),
        )
        .unwrap();

        // Store a proof
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            data_hash: DATA_HASH.to_string(),
            original_data_reference: Some(
                "ipfs://QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco".to_string(),
            ),
            data_owner: Some(USER.to_string()),
            metadata_json: Some(r#"{"facility_id": "F123", "device_id": "D456"}"#.to_string()),
            tw_start: Timestamp::from_nanos(0),
            tw_end: Timestamp::from_nanos(0),
            value_in: None,
            value_out: None,
            unit: "kWh".to_string(),
        });

        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &store_msg,
            &[],
        )
        .unwrap();

        // Verify the proof was stored
        let query_msg = QueryMsg::ProofByHash {
            data_hash: DATA_HASH.to_string(),
        };
        let proof: ProofResponse = app
            .wrap()
            .query_wasm_smart(contract_addr, &query_msg)
            .unwrap();

        assert_eq!(proof.data_hash, DATA_HASH.to_string());
        assert_eq!(proof.stored_by, Addr::unchecked(USER));
    }

    #[test]
    fn test_admin_operations() {
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());

        let instantiate_msg = default_instantiate_msg(); // use_whitelist is true by default
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked(ADMIN),
                &instantiate_msg,
                &[],
                "DeTrack",
                None,
            )
            .unwrap();

        // Whitelist a node (NODE_USER)
        let whitelist_msg = ExecuteMsg::Admin(AdminExecuteMsg::WhitelistNode {
            node_address: NODE_USER.to_string(),
        });

        app.execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &whitelist_msg,
            &[],
        )
        .unwrap();

        // Verify the node is whitelisted
        let query_msg = QueryMsg::IsWhitelisted {
            address: NODE_USER.to_string(),
        };
        let whitelist_response: WhitelistedResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &query_msg)
            .unwrap();

        assert!(whitelist_response.is_whitelisted);

        // Update node reputation (assuming Node object exists for whitelisted node or is created)
        // This part might need adjustment based on whether whitelisted nodes automatically get a Node entry
        // or if UpdateNodeReputation creates one / applies to registered nodes only.
        // For now, assuming it works for a whitelisted node if an entry is implicitly created or not strictly required for this call.
        let update_reputation_msg = ExecuteMsg::Admin(AdminExecuteMsg::UpdateNodeReputation {
            node_address: NODE_USER.to_string(),
            reputation: 10,
        });

        app.execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &update_reputation_msg,
            &[],
        )
        .unwrap();

        // Verify the reputation was updated
        // This query might fail if NODE_USER doesn't have a full Node entry from just being whitelisted.
        // The summary mentioned Node struct has proof_count etc. Whitelisting might not populate all of that.
        // Let's assume QueryMsg::NodeReputation { address: NODE_USER.to_string() } works.
        let query_rep_msg = QueryMsg::NodeReputation { address: NODE_USER.to_string() };
        let reputation_response: NodeReputationResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &query_rep_msg)
            .unwrap();

        assert_eq!(reputation_response.reputation, 10);

        // Update min reputation threshold
        let new_threshold: i32 = 50; // Changed type to i32
        let update_threshold_msg = ExecuteMsg::Admin(AdminExecuteMsg::UpdateMinReputationThreshold {
            threshold: new_threshold,
        });

        app.execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &update_threshold_msg,
            &[],
        )
        .unwrap();

        // Verify the threshold was updated
        let query_cfg_msg = QueryMsg::Config {};
        let config_response: ConfigResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &query_cfg_msg)
            .unwrap();

        assert_eq!(config_response.min_reputation_threshold, new_threshold);
    }

    #[test]
    fn test_unauthorized_access() {
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());

        // Instantiate with use_whitelist = true (default)
        let instantiate_msg = default_instantiate_msg();
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked(ADMIN),
                &instantiate_msg,
                &[],
                "DeTrack",
                None,
            )
            .unwrap();

        // USER (non-admin) tries to perform admin operation (WhitelistNode)
        let whitelist_msg = ExecuteMsg::Admin(AdminExecuteMsg::WhitelistNode {
            node_address: "someone_else".to_string(),
        });

        let err = app
            .execute_contract(
                Addr::unchecked(USER), // USER is not ADMIN
                contract_addr.clone(),
                &whitelist_msg,
                &[],
            )
            .unwrap_err();

        // Check for specific admin-only error or a general unauthorized
        // The original test checked for "Admin only operation". This depends on ContractError enum.
        // Assuming ContractError::Unauthorized or a similar specific error like AdminOnly.
        assert!(matches!(err.downcast_ref::<ContractError>().unwrap(), ContractError::AdminOnlyOperation {}));

        // USER2 (not whitelisted) tries to store proof when use_whitelist is true
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            data_hash: DATA_HASH.to_string(),
            original_data_reference: None,
            data_owner: None,
            metadata_json: None,
            tw_start: Timestamp::from_nanos(0), // Added
            tw_end: Timestamp::from_nanos(0), // Added
            value_in: None, // Added
            value_out: None, // Added
            unit: "kWh".to_string(), // Added
        });

        let err_store = app
            .execute_contract(
                Addr::unchecked(USER2), // USER2 is not whitelisted
                contract_addr.clone(),
                &store_msg,
                &[],
            )
            .unwrap_err();

        assert!(matches!(err_store.downcast_ref::<ContractError>().unwrap(), ContractError::NodeNotWhitelisted(ref addr) if addr == USER2));
    }

    #[test]
    fn test_unauthorized_access_when_use_whitelist_is_false() {
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());

        let mut instantiate_msg = default_instantiate_msg();
        instantiate_msg.use_whitelist = false;

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked(ADMIN),
                &instantiate_msg,
                &[],
                "DeTrack",
                None,
            )
            .unwrap();

        // USER2 (non-admin) tries admin operation
        let set_whitelist_mode_msg = ExecuteMsg::Admin(AdminExecuteMsg::UpdateAdmin { new_admin: USER2.to_string() }); // Changed to a valid AdminExecuteMsg
        let err_admin_op = app.execute_contract(Addr::unchecked(USER2), contract_addr.clone(), &set_whitelist_mode_msg, &[]).unwrap_err();
        assert!(matches!(err_admin_op.downcast_ref::<ContractError>().unwrap(), ContractError::AdminOnlyOperation {})); // Changed to AdminOnlyOperation

        // USER (not registered) tries to store proof
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            data_hash: "some_hash".to_string(),
            original_data_reference: None, data_owner: None, metadata_json: None,
            tw_start: Timestamp::from_nanos(0), // Added
            tw_end: Timestamp::from_nanos(0), // Added
            value_in: None, // Added
            value_out: None, // Added
            unit: "kWh".to_string(), // Added
        });
        let err_store = app.execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[]).unwrap_err();
        assert!(matches!(err_store.downcast_ref::<ContractError>().unwrap(), ContractError::NodeNotWhitelisted(ref addr) if addr == USER), "Expected NodeNotWhitelisted error, got {:?}", err_store);

        // USER stakes and registers
        // app.staking_delegate( // Removed staking_delegate call
        //     &Addr::unchecked(USER),
        //     &Addr::unchecked(VALIDATOR),
        //     Coin::new(instantiate_msg.min_stake_tier1.u128(), NATIVE_DENOM),
        // ).unwrap();
        let register_msg = ExecuteMsg::Node(NodeExecuteMsg::RegisterNode {});
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &register_msg,
            &coins(instantiate_msg.deposit_tier1.u128(), NATIVE_DENOM),
        ).unwrap();

        // USER (now registered) tries to store proof -> should succeed
        app.execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[]).unwrap();
    }

    #[test]
    fn test_deposit_operations() {
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());
        let mut instantiate_msg = default_instantiate_msg();
        instantiate_msg.use_whitelist = false; // Nodes will register directly

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked(ADMIN),
                &instantiate_msg,
                &[],
                "DeTrack",
                None,
            )
            .unwrap();

        let node_addr = Addr::unchecked(NODE_USER);
        let initial_deposit_amount = instantiate_msg.deposit_tier1;
        let additional_deposit_amount = Uint128::new(50);

        // 1. Register Node (NODE_USER)
        // Stake enough for Tier 1
        // app.staking_delegate( // Removed staking_delegate call
        //     &node_addr,
        //     &Addr::unchecked(VALIDATOR),
        //     Coin::new(instantiate_msg.min_stake_tier1.u128(), NATIVE_DENOM),
        // )
        // .unwrap();

        let register_msg = ExecuteMsg::Node(NodeExecuteMsg::RegisterNode {});
        app.execute_contract(
            node_addr.clone(),
            contract_addr.clone(),
            &register_msg,
            &coins(initial_deposit_amount.u128(), NATIVE_DENOM),
        )
        .unwrap();

        // Verify initial deposit
        let node_info: NodeInfoResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::NodeInfo { address: node_addr.to_string() },
            )
            .unwrap();
        assert_eq!(node_info.deposit, Some(initial_deposit_amount));

        // 2. Add Deposit
        let add_deposit_msg = ExecuteMsg::Node(NodeExecuteMsg::AddDeposit {});
        app.execute_contract(
            node_addr.clone(),
            contract_addr.clone(),
            &add_deposit_msg,
            &coins(additional_deposit_amount.u128(), NATIVE_DENOM),
        )
        .unwrap();

        let node_info_after_add: NodeInfoResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::NodeInfo {
                    address: node_addr.to_string(),
                },
            )
            .unwrap();
        assert_eq!(
            node_info_after_add.deposit,
            Some(initial_deposit_amount + additional_deposit_amount)
        );

        // 3. Unlock Deposit
        let unlock_deposit_msg = ExecuteMsg::Node(NodeExecuteMsg::UnlockDeposit {});
        app.execute_contract(
            node_addr.clone(),
            contract_addr.clone(),
            &unlock_deposit_msg,
            &[],
        )
        .unwrap();

        // Verify node\'s active deposit is now zero
        let node_info_after_unlock_init: NodeInfoResponse = app
            .wrap()
            .query_wasm_smart(
                contract_addr.clone(),
                &QueryMsg::NodeInfo { address: node_addr.to_string() },
            )
            .unwrap();
        assert_eq!(node_info_after_unlock_init.deposit, Some(Uint128::zero()));

        // Try to add deposit while unlocking -> should fail
        let err_add_while_unlocking = app
            .execute_contract(
                node_addr.clone(),
                contract_addr.clone(),
                &add_deposit_msg,
                &coins(additional_deposit_amount.u128(), NATIVE_DENOM),
            )
            .unwrap_err();
        assert!(matches!(
            err_add_while_unlocking.downcast_ref::<ContractError>().unwrap(),
            ContractError::DepositAlreadyUnlocking {}
        ));

        // 4. Claim Unlocked Deposit
        // Advance blocks to pass the unlocking period
        app.update_block(|block| {
            block.height += instantiate_msg.deposit_unlock_period_blocks;
        });

        let claim_deposit_msg = ExecuteMsg::Node(NodeExecuteMsg::ClaimUnlockedDeposit {});
        let balance_before_claim = app.wrap().query_balance(&node_addr, NATIVE_DENOM).unwrap().amount;

        app.execute_contract(
            node_addr.clone(),
            contract_addr.clone(),
            &claim_deposit_msg,
            &[],
        )
        .unwrap();

        // Verify deposit is claimed (node\'s balance increased)
        let balance_after_claim = app.wrap().query_balance(&node_addr, NATIVE_DENOM).unwrap().amount;
        assert_eq!(
            balance_after_claim,
            balance_before_claim + initial_deposit_amount + additional_deposit_amount
        );

        // Verify UnlockingDeposits entry is removed (cannot query directly without a specific query message)
        // Attempting to claim again should fail
        let err_claim_again = app
            .execute_contract(
                node_addr.clone(),
                contract_addr.clone(),
                &claim_deposit_msg,
                &[],
            )
            .unwrap_err();
        assert!(matches!(
            err_claim_again.downcast_ref::<ContractError>().unwrap(),
            ContractError::NoUnlockedDepositToClaim {}
        ));
    }
}
