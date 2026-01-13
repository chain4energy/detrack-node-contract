#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, coins, Empty, Uint128, Timestamp};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    use crate::contract::{execute, instantiate, query};
    use crate::msg::{
        ExecuteMsg, InstantiateMsg, QueryMsg, ConfigResponse, ProofResponse, ProofsResponse, NodeExecuteMsg,
        AdminExecuteMsg, NodeInfoResponse, WhitelistedResponse, NodeReputationResponse,
        BatchInfo,
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
            did_contract_address: "c4e1qkphn8h2rnyqjjtfh8j8dtuqgh5cac57nq2286tsljducqp4lwfqvsysy0".to_string(),
            version: "1.0.0".to_string(),
            min_stake_tier1: Uint128::new(1000),
            min_stake_tier2: Uint128::new(5000),
            min_stake_tier3: Uint128::new(10000),
            deposit_tier1: Uint128::new(100), // uc4e
            deposit_tier2: Uint128::new(500), // uc4e
            deposit_tier3: Uint128::new(1000), // uc4e
            use_whitelist: true,
            deposit_unlock_period_blocks: 100,
            max_batch_size: 100, // Default maximum batch size
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

        // Store a proof (Phase 1b format)
        let batch_metadata = vec![BatchInfo {
            batch_id: "batch-001".to_string(),
            gateway_did: r"did:c4e:gateway:test-gw1".to_string(),
            device_count: 5,
            snapshot_count: 10,
            batch_merkle_root: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        }];
        
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: DATA_HASH.to_string(),
            tw_start: Timestamp::from_nanos(1704067200000000000), // 2024-01-01 00:00:00 UTC
            tw_end: Timestamp::from_nanos(1704153600000000000),   // 2024-01-02 00:00:00 UTC
            batch_metadata,
            metadata_json: Some(r#"{"facility_id": "F123", "device_id": "D456"}"#.to_string()),
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

        // USER2 (not whitelisted) tries to store proof when use_whitelist is true (Phase 1b format)
        let batch_metadata = vec![BatchInfo {
            batch_id: "batch-002".to_string(),
            gateway_did: r"did:c4e:gateway:test-gw2".to_string(),
            device_count: 3,
            snapshot_count: 6,
            batch_merkle_root: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        }];
        
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack2".to_string(),
            data_hash: DATA_HASH.to_string(),
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata,
            metadata_json: None,
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

        // USER (not registered) tries to store proof (Phase 1b format)
        let batch_metadata = vec![BatchInfo {
            batch_id: "batch-003".to_string(),
            gateway_did: r"did:c4e:gateway:test-gw3".to_string(),
            device_count: 2,
            snapshot_count: 4,
            batch_merkle_root: "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210".to_string(),
        }];
        
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack3".to_string(),
            data_hash: "abcd1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab".to_string(), // Valid 64-char hex
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata,
            metadata_json: None,
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

    // =========================================================================
    // COMPREHENSIVE STORE_PROOF TESTS
    // =========================================================================

    #[test]
    fn test_store_proof_error_empty_batch_metadata() {
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());
        let instantiate_msg = default_instantiate_msg();
        let contract_addr = app
            .instantiate_contract(contract_id, Addr::unchecked(ADMIN), &instantiate_msg, &[], "DeTrack", None)
            .unwrap();

        // Register node
        let register_msg = ExecuteMsg::Node(NodeExecuteMsg::RegisterNode {});
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &register_msg,
            &coins(instantiate_msg.deposit_tier1.u128(), NATIVE_DENOM),
        )
        .unwrap();

        // Try to store proof with empty batch_metadata
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: DATA_HASH.to_string(),
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata: vec![], // EMPTY
            metadata_json: None,
        });

        let err = app
            .execute_contract(Addr::unchecked(USER), contract_addr, &store_msg, &[])
            .unwrap_err();

        assert!(matches!(
            err.downcast_ref::<ContractError>().unwrap(),
            ContractError::EmptyBatchMetadata {}
        ));
    }

    #[test]
    fn test_store_proof_error_too_many_batches() {
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());
        let instantiate_msg = default_instantiate_msg();
        let contract_addr = app
            .instantiate_contract(contract_id, Addr::unchecked(ADMIN), &instantiate_msg, &[], "DeTrack", None)
            .unwrap();

        // Register node
        let register_msg = ExecuteMsg::Node(NodeExecuteMsg::RegisterNode {});
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &register_msg,
            &coins(instantiate_msg.deposit_tier1.u128(), NATIVE_DENOM),
        )
        .unwrap();

        // Create 101 batches (over limit)
        let batch_metadata: Vec<BatchInfo> = (0..101)
            .map(|i| BatchInfo {
                batch_id: format!("batch-{:03}", i),
                gateway_did: format!("did:c4e:gateway:gw{}", i % 5),
                device_count: 5,
                snapshot_count: 10,
                batch_merkle_root: format!("{:0<64}", format!("{:x}", i)),
            })
            .collect();

        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: DATA_HASH.to_string(),
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata,
            metadata_json: None,
        });

        let err = app
            .execute_contract(Addr::unchecked(USER), contract_addr, &store_msg, &[])
            .unwrap_err();

        assert!(matches!(
            err.downcast_ref::<ContractError>().unwrap(),
            ContractError::TooManyBatches { count: 101 }
        ));
    }

    #[test]
    fn test_store_proof_error_invalid_data_hash() {
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());
        let instantiate_msg = default_instantiate_msg();
        let contract_addr = app
            .instantiate_contract(contract_id, Addr::unchecked(ADMIN), &instantiate_msg, &[], "DeTrack", None)
            .unwrap();

        // Register node
        let register_msg = ExecuteMsg::Node(NodeExecuteMsg::RegisterNode {});
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &register_msg,
            &coins(instantiate_msg.deposit_tier1.u128(), NATIVE_DENOM),
        )
        .unwrap();

        let batch_metadata = vec![BatchInfo {
            batch_id: "batch-001".to_string(),
            gateway_did: r"did:c4e:gateway:test-gw1".to_string(),
            device_count: 5,
            snapshot_count: 10,
            batch_merkle_root: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        }];

        // Test 1: Empty data_hash
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: "".to_string(), // EMPTY
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata: batch_metadata.clone(),
            metadata_json: None,
        });

        let err = app
            .execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[])
            .unwrap_err();

        assert!(matches!(
            err.downcast_ref::<ContractError>().unwrap(),
            ContractError::InvalidInput(_)
        ));

        // Test 2: Invalid length (not 64 chars)
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: "abc123".to_string(), // TOO SHORT
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata: batch_metadata.clone(),
            metadata_json: None,
        });

        let err = app
            .execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[])
            .unwrap_err();

        assert!(matches!(
            err.downcast_ref::<ContractError>().unwrap(),
            ContractError::InvalidInput(_)
        ));

        // Test 3: Invalid characters (not hex)
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: "ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ".to_string(), // INVALID HEX
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata,
            metadata_json: None,
        });

        let err = app
            .execute_contract(Addr::unchecked(USER), contract_addr, &store_msg, &[])
            .unwrap_err();

        assert!(matches!(
            err.downcast_ref::<ContractError>().unwrap(),
            ContractError::InvalidInput(_)
        ));
    }

    #[test]
    fn test_store_proof_error_proof_already_exists() {
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());
        let instantiate_msg = default_instantiate_msg();
        let contract_addr = app
            .instantiate_contract(contract_id, Addr::unchecked(ADMIN), &instantiate_msg, &[], "DeTrack", None)
            .unwrap();

        // Register node
        let register_msg = ExecuteMsg::Node(NodeExecuteMsg::RegisterNode {});
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &register_msg,
            &coins(instantiate_msg.deposit_tier1.u128(), NATIVE_DENOM),
        )
        .unwrap();

        let batch_metadata = vec![BatchInfo {
            batch_id: "batch-001".to_string(),
            gateway_did: r"did:c4e:gateway:test-gw1".to_string(),
            device_count: 5,
            snapshot_count: 10,
            batch_merkle_root: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        }];

        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: DATA_HASH.to_string(),
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata: batch_metadata.clone(),
            metadata_json: None,
        });

        // First submission - should succeed
        app.execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[])
            .unwrap();

        // Second submission with same data_hash - should fail
        let err = app
            .execute_contract(Addr::unchecked(USER), contract_addr, &store_msg, &[])
            .unwrap_err();

        assert!(matches!(
            err.downcast_ref::<ContractError>().unwrap(),
            ContractError::ProofAlreadyExists(_)
        ));
    }

    #[test]
    fn test_store_proof_error_invalid_did_format() {
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());
        let instantiate_msg = default_instantiate_msg();
        let contract_addr = app
            .instantiate_contract(contract_id, Addr::unchecked(ADMIN), &instantiate_msg, &[], "DeTrack", None)
            .unwrap();

        // Register node
        let register_msg = ExecuteMsg::Node(NodeExecuteMsg::RegisterNode {});
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &register_msg,
            &coins(instantiate_msg.deposit_tier1.u128(), NATIVE_DENOM),
        )
        .unwrap();

        // Test 1: Invalid worker_did format
        let batch_metadata = vec![BatchInfo {
            batch_id: "batch-001".to_string(),
            gateway_did: r"did:c4e:gateway:test-gw1".to_string(),
            device_count: 5,
            snapshot_count: 10,
            batch_merkle_root: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        }];

        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: "invalid-did-format".to_string(), // INVALID
            data_hash: DATA_HASH.to_string(),
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata: batch_metadata.clone(),
            metadata_json: None,
        });

        let err = app
            .execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[])
            .unwrap_err();

        assert!(matches!(
            err.downcast_ref::<ContractError>().unwrap(),
            ContractError::InvalidDidFormat { .. }
        ));

        // Test 2: Invalid gateway_did format
        let batch_metadata = vec![BatchInfo {
            batch_id: "batch-001".to_string(),
            gateway_did: "not-a-did".to_string(), // INVALID
            device_count: 5,
            snapshot_count: 10,
            batch_merkle_root: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        }];

        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata,
            metadata_json: None,
        });

        let err = app
            .execute_contract(Addr::unchecked(USER), contract_addr, &store_msg, &[])
            .unwrap_err();

        assert!(matches!(
            err.downcast_ref::<ContractError>().unwrap(),
            ContractError::InvalidDidFormat { .. }
        ));
    }

    #[test]
    fn test_store_proof_events_emitted() {
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());
        let instantiate_msg = default_instantiate_msg();
        let contract_addr = app
            .instantiate_contract(contract_id, Addr::unchecked(ADMIN), &instantiate_msg, &[], "DeTrack", None)
            .unwrap();

        // Register node
        let register_msg = ExecuteMsg::Node(NodeExecuteMsg::RegisterNode {});
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &register_msg,
            &coins(instantiate_msg.deposit_tier1.u128(), NATIVE_DENOM),
        )
        .unwrap();

        let batch_metadata = vec![
            BatchInfo {
                batch_id: "batch-001".to_string(),
                gateway_did: r"did:c4e:gateway:test-gw1".to_string(),
                device_count: 5,
                snapshot_count: 10,
                batch_merkle_root: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            },
            BatchInfo {
                batch_id: "batch-002".to_string(),
                gateway_did: r"did:c4e:gateway:test-gw2".to_string(),
                device_count: 3,
                snapshot_count: 8,
                batch_merkle_root: "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210".to_string(),
            },
        ];

        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: DATA_HASH.to_string(),
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata: batch_metadata.clone(),
            metadata_json: Some(r#"{"test": "metadata"}"#.to_string()),
        });

        let res = app
            .execute_contract(Addr::unchecked(USER), contract_addr, &store_msg, &[])
            .unwrap();

        // Verify events
        let store_proof_event = res.events.iter().find(|e| e.ty == "wasm-store_proof").unwrap();
        
        assert_eq!(
            store_proof_event.attributes.iter().find(|a| a.key == "action").unwrap().value,
            "store_proof"
        );
        assert_eq!(
            store_proof_event.attributes.iter().find(|a| a.key == "proof_id").unwrap().value,
            "0"
        );
        assert_eq!(
            store_proof_event.attributes.iter().find(|a| a.key == "worker_did").unwrap().value,
            r"did:c4e:worker:detrack1"
        );
        assert_eq!(
            store_proof_event.attributes.iter().find(|a| a.key == "data_hash").unwrap().value,
            DATA_HASH
        );
        assert_eq!(
            store_proof_event.attributes.iter().find(|a| a.key == "stored_by").unwrap().value,
            USER
        );
        assert_eq!(
            store_proof_event.attributes.iter().find(|a| a.key == "batch_count").unwrap().value,
            "2"
        );
        
        let gateway_dids = store_proof_event.attributes.iter().find(|a| a.key == "gateway_dids").unwrap().value.as_str();
        assert!(gateway_dids.contains("did:c4e:gateway:test-gw1"));
        assert!(gateway_dids.contains("did:c4e:gateway:test-gw2"));
    }

    #[test]
    fn test_store_proof_logic_and_indexes() {
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());
        let instantiate_msg = default_instantiate_msg();
        let contract_addr = app
            .instantiate_contract(contract_id, Addr::unchecked(ADMIN), &instantiate_msg, &[], "DeTrack", None)
            .unwrap();

        // Register node
        let register_msg = ExecuteMsg::Node(NodeExecuteMsg::RegisterNode {});
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &register_msg,
            &coins(instantiate_msg.deposit_tier1.u128(), NATIVE_DENOM),
        )
        .unwrap();

        let batch_metadata = vec![
            BatchInfo {
                batch_id: "batch-001".to_string(),
                gateway_did: r"did:c4e:gateway:test-gw1".to_string(),
                device_count: 5,
                snapshot_count: 10,
                batch_merkle_root: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            },
            BatchInfo {
                batch_id: "batch-002".to_string(),
                gateway_did: r"did:c4e:gateway:test-gw2".to_string(),
                device_count: 3,
                snapshot_count: 8,
                batch_merkle_root: "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210".to_string(),
            },
        ];

        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: DATA_HASH.to_string(),
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata: batch_metadata.clone(),
            metadata_json: Some(r#"{"facility_id": "F123"}"#.to_string()),
        });

        app.execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[])
            .unwrap();

        // Test 1: Query by proof ID
        let query_msg = QueryMsg::Proof { id: 0 };
        let proof: ProofResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &query_msg).unwrap();
        
        assert_eq!(proof.id, 0);
        assert_eq!(proof.worker_did, r"did:c4e:worker:detrack1");
        assert_eq!(proof.data_hash, DATA_HASH);
        assert_eq!(proof.batch_metadata.len(), 2);
        assert_eq!(proof.tw_start, Timestamp::from_nanos(1704067200000000000));
        assert_eq!(proof.tw_end, Timestamp::from_nanos(1704153600000000000));

        // Test 2: Query by data hash (index)
        let query_msg = QueryMsg::ProofByHash { data_hash: DATA_HASH.to_string() };
        let proof: ProofResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &query_msg).unwrap();
        assert_eq!(proof.id, 0);

        // Test 3: Query by worker DID (secondary index)
        let query_msg = QueryMsg::ProofsByWorker {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            start_after: None,
            limit: None,
        };
        let proofs: ProofsResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &query_msg).unwrap();
        assert_eq!(proofs.proofs.len(), 1);
        assert_eq!(proofs.proofs[0].id, 0);

        // Test 4: Query by gateway DID (manual index)
        let query_msg = QueryMsg::ProofsByGateway {
            gateway_did: r"did:c4e:gateway:test-gw1".to_string(),
            start_after: None,
            limit: None,
        };
        let proofs: ProofsResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &query_msg).unwrap();
        assert_eq!(proofs.proofs.len(), 1);

        let query_msg = QueryMsg::ProofsByGateway {
            gateway_did: r"did:c4e:gateway:test-gw2".to_string(),
            start_after: None,
            limit: None,
        };
        let proofs: ProofsResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &query_msg).unwrap();
        assert_eq!(proofs.proofs.len(), 1);

        // Test 5: Verify proof_count incremented
        let query_msg = QueryMsg::Config {};
        let config: ConfigResponse = app.wrap().query_wasm_smart(contract_addr, &query_msg).unwrap();
        assert_eq!(config.proof_count, 1);
    }

    #[test]
    fn test_store_proof_multi_gateway_real_world() {
        // Real-world test: 21 batches, 3 gateways (from production payload)
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());
        let instantiate_msg = default_instantiate_msg();
        let contract_addr = app
            .instantiate_contract(contract_id, Addr::unchecked(ADMIN), &instantiate_msg, &[], "DeTrack", None)
            .unwrap();

        // Register node
        let register_msg = ExecuteMsg::Node(NodeExecuteMsg::RegisterNode {});
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &register_msg,
            &coins(instantiate_msg.deposit_tier1.u128(), NATIVE_DENOM),
        )
        .unwrap();

        // Build 21 batches matching production payload structure
        let batch_metadata = vec![
            // Gateway 1: 12 batches
            BatchInfo { batch_id: "batch-1768245621345-c6f60c37".to_string(), gateway_did: r"did:c4e:gateway:test-gw1".to_string(), device_count: 6, snapshot_count: 6, batch_merkle_root: "b22254af00d894091755eec8bd50a0bcfb83633aed5d7323154850de5bc2722a".to_string() },
            BatchInfo { batch_id: "batch-1768245626346-460e0c3e".to_string(), gateway_did: r"did:c4e:gateway:test-gw1".to_string(), device_count: 6, snapshot_count: 6, batch_merkle_root: "8d227d7640f62a291adbad2b002a755e2a611c846885c5c6a33ced7595b9a95e".to_string() },
            BatchInfo { batch_id: "batch-1768245631347-5afb1e5a".to_string(), gateway_did: r"did:c4e:gateway:test-gw1".to_string(), device_count: 6, snapshot_count: 6, batch_merkle_root: "cd70e8d0f13beb8d62eb20589047d0256d5551f9bb917a76bd2b91fe5d92fcd5".to_string() },
            BatchInfo { batch_id: "batch-1768245636347-500930fa".to_string(), gateway_did: r"did:c4e:gateway:test-gw1".to_string(), device_count: 6, snapshot_count: 6, batch_merkle_root: "062efc63e9469f03d151d79096f58113c783787467d403a9d747c72ae3092a19".to_string() },
            BatchInfo { batch_id: "batch-1768245641347-97c9a268".to_string(), gateway_did: r"did:c4e:gateway:test-gw1".to_string(), device_count: 6, snapshot_count: 6, batch_merkle_root: "bd7a7856d31bea65f3db9a396990e65cf9a8512e191fc134268652c265549e1e".to_string() },
            BatchInfo { batch_id: "batch-1768245646350-91409bca".to_string(), gateway_did: r"did:c4e:gateway:test-gw1".to_string(), device_count: 6, snapshot_count: 6, batch_merkle_root: "23d65b9f4ca7701c144b9b9569543a73d42d86c4e7bbe19f05cb6461e242fe1a".to_string() },
            BatchInfo { batch_id: "batch-1768245651350-472dfbc8".to_string(), gateway_did: r"did:c4e:gateway:test-gw1".to_string(), device_count: 6, snapshot_count: 6, batch_merkle_root: "28c12c02973bb5d569fea44034f3e26ac4b4d521b77e48a07c8731bb8849eb39".to_string() },
            BatchInfo { batch_id: "batch-1768245656352-ddd9d741".to_string(), gateway_did: r"did:c4e:gateway:test-gw1".to_string(), device_count: 6, snapshot_count: 6, batch_merkle_root: "606b19cf80deebadbe17a5b24243e98cf806fc9bc36dadc269523a229cf60cac".to_string() },
            BatchInfo { batch_id: "batch-1768245661353-be8ead6c".to_string(), gateway_did: r"did:c4e:gateway:test-gw1".to_string(), device_count: 6, snapshot_count: 6, batch_merkle_root: "176fc29e6da1d82868203531b32f0ad4ebcf2d21a96677b5f425fb0a297784ab".to_string() },
            BatchInfo { batch_id: "batch-1768245666355-ac828677".to_string(), gateway_did: r"did:c4e:gateway:test-gw1".to_string(), device_count: 6, snapshot_count: 6, batch_merkle_root: "11e9cb449d5f91fb66b1197076a9babb1199a47a56d051b385741ee77dd26406".to_string() },
            BatchInfo { batch_id: "batch-1768245671356-b9e5605b".to_string(), gateway_did: r"did:c4e:gateway:test-gw1".to_string(), device_count: 6, snapshot_count: 6, batch_merkle_root: "39319004af7807df85ac14fd26f11792f7820b6fba29005b846101a072d3fd85".to_string() },
            BatchInfo { batch_id: "batch-1768245676358-371f382d".to_string(), gateway_did: r"did:c4e:gateway:test-gw1".to_string(), device_count: 6, snapshot_count: 6, batch_merkle_root: "cba7969c2428cacde1a2a2b99397799f764cdfae7df2647b451bb8133cfb51e4".to_string() },
            // Gateway 3: 3 batches
            BatchInfo { batch_id: "batch-1768245624806-bc4c0546".to_string(), gateway_did: r"did:c4e:gateway:test-gw3".to_string(), device_count: 14, snapshot_count: 14, batch_merkle_root: "78896cdc433130eaf5bfa19809ceff9fb0975b6fb8a993f91638fd6bb55c2264".to_string() },
            BatchInfo { batch_id: "batch-1768245639807-68f397de".to_string(), gateway_did: r"did:c4e:gateway:test-gw3".to_string(), device_count: 14, snapshot_count: 14, batch_merkle_root: "4a856c6f1ea18dec74bd847f4bcf682cb29ef1d5cfd85a9d35691134eb367c2c".to_string() },
            BatchInfo { batch_id: "batch-1768245669817-8a7b0272".to_string(), gateway_did: r"did:c4e:gateway:test-gw3".to_string(), device_count: 14, snapshot_count: 14, batch_merkle_root: "77d5d48b2b82ec8f82ad46de1a14619da3248222d713b6685a95d0e4d9778a9c".to_string() },
            // Gateway 2: 6 batches
            BatchInfo { batch_id: "batch-1768245627876-e18d8098".to_string(), gateway_did: r"did:c4e:gateway:test-gw2".to_string(), device_count: 10, snapshot_count: 10, batch_merkle_root: "8fbe904d674ae8f772af45f859569e0f9c2e5cd50c93f6407bf6c27880185a45".to_string() },
            BatchInfo { batch_id: "batch-1768245637877-a0d51b29".to_string(), gateway_did: r"did:c4e:gateway:test-gw2".to_string(), device_count: 10, snapshot_count: 10, batch_merkle_root: "24718a64db6d1a55f3347989f445e27da230c8b0dd6b27302ab9c702628c275e".to_string() },
            BatchInfo { batch_id: "batch-1768245647883-9fc58403".to_string(), gateway_did: r"did:c4e:gateway:test-gw2".to_string(), device_count: 10, snapshot_count: 10, batch_merkle_root: "c231832c8ee2b6526294b09c79f36b65d144ca07c87028771eeb45e4026b64df".to_string() },
            BatchInfo { batch_id: "batch-1768245657887-5074480f".to_string(), gateway_did: r"did:c4e:gateway:test-gw2".to_string(), device_count: 10, snapshot_count: 10, batch_merkle_root: "bfc3f534f2af13a9ee2f8dcec9cc5eee39608a9e25102fd29bf1b71651415b01".to_string() },
            BatchInfo { batch_id: "batch-1768245667887-0775c607".to_string(), gateway_did: r"did:c4e:gateway:test-gw2".to_string(), device_count: 10, snapshot_count: 10, batch_merkle_root: "532cca7ba8145d5f816d2557cd0a3ea28787e7f9475b359a2973caa4d4740d97".to_string() },
            BatchInfo { batch_id: "batch-1768245677893-834db962".to_string(), gateway_did: r"did:c4e:gateway:test-gw2".to_string(), device_count: 10, snapshot_count: 10, batch_merkle_root: "1278a9833249bf41e92843ba2505a63184d1487226142467667bc97ae3dd0f74".to_string() },
        ];

        // Gateway metadata as metadata_json (not in contract schema)
        let metadata_json = r#"{
            "aggregation_strategy": "phase2_multi_gateway",
            "worker_version": "0.2.0",
            "gateway_count": 3,
            "total_snapshot_count": 174,
            "gateway_metadata": [
                {"gateway_did": "did:c4e:gateway:test-gw1", "batch_count": 12, "snapshot_count": 72},
                {"gateway_did": "did:c4e:gateway:test-gw3", "batch_count": 3, "snapshot_count": 42},
                {"gateway_did": "did:c4e:gateway:test-gw2", "batch_count": 6, "snapshot_count": 60}
            ]
        }"#;

        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack2".to_string(),
            data_hash: "968a30bd6b874139315a6cc1c45ae7d837695e3de9aa1ee471f133cd4e3035bc".to_string(),
            tw_start: Timestamp::from_nanos(1768245621344000000),
            tw_end: Timestamp::from_nanos(1768245677893000000),
            batch_metadata,
            metadata_json: Some(metadata_json.to_string()),
        });

        let res = app
            .execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[])
            .unwrap();

        // Verify event
        let store_proof_event = res.events.iter().find(|e| e.ty == "wasm-store_proof").unwrap();
        assert_eq!(
            store_proof_event.attributes.iter().find(|a| a.key == "batch_count").unwrap().value,
            "21"
        );

        // Query proof
        let query_msg = QueryMsg::Proof { id: 0 };
        let proof: ProofResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &query_msg).unwrap();
        assert_eq!(proof.batch_metadata.len(), 21);
        assert_eq!(proof.worker_did, r"did:c4e:worker:detrack2");

        // Verify all 3 gateways are indexed
        let query_msg = QueryMsg::ProofsByGateway {
            gateway_did: r"did:c4e:gateway:test-gw1".to_string(),
            start_after: None,
            limit: None,
        };
        let proofs: ProofsResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &query_msg).unwrap();
        assert_eq!(proofs.proofs.len(), 1);

        let query_msg = QueryMsg::ProofsByGateway {
            gateway_did: r"did:c4e:gateway:test-gw2".to_string(),
            start_after: None,
            limit: None,
        };
        let proofs: ProofsResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &query_msg).unwrap();
        assert_eq!(proofs.proofs.len(), 1);

        let query_msg = QueryMsg::ProofsByGateway {
            gateway_did: r"did:c4e:gateway:test-gw3".to_string(),
            start_after: None,
            limit: None,
        };
        let proofs: ProofsResponse = app.wrap().query_wasm_smart(contract_addr, &query_msg).unwrap();
        assert_eq!(proofs.proofs.len(), 1);
    }

    // =========================================================================
    // P0: TIME WINDOW VALIDATION TESTS
    // =========================================================================

    #[test]
    fn test_time_window_valid_ranges() {
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());
        let instantiate_msg = default_instantiate_msg();
        let contract_addr = app
            .instantiate_contract(contract_id, Addr::unchecked(ADMIN), &instantiate_msg, &[], "DeTrack", None)
            .unwrap();

        // Register node
        let register_msg = ExecuteMsg::Node(NodeExecuteMsg::RegisterNode {});
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &register_msg,
            &coins(instantiate_msg.deposit_tier1.u128(), NATIVE_DENOM),
        )
        .unwrap();

        let batch_metadata = vec![BatchInfo {
            batch_id: "batch-001".to_string(),
            gateway_did: r"did:c4e:gateway:test-gw1".to_string(),
            device_count: 5,
            snapshot_count: 10,
            batch_merkle_root: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        }];

        // Test 1: Zero timestamp (epoch start)
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: "1111111111111111111111111111111111111111111111111111111111111111".to_string(),
            tw_start: Timestamp::from_nanos(0),
            tw_end: Timestamp::from_nanos(1000000000),
            batch_metadata: batch_metadata.clone(),
            metadata_json: None,
        });
        app.execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[])
            .unwrap();

        // Test 2: Same start and end (instant)
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: "2222222222222222222222222222222222222222222222222222222222222222".to_string(),
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704067200000000000),
            batch_metadata: batch_metadata.clone(),
            metadata_json: None,
        });
        app.execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[])
            .unwrap();

        // Test 3: Very large timestamps (far future - year 2050+)
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: "3333333333333333333333333333333333333333333333333333333333333333".to_string(),
            tw_start: Timestamp::from_nanos(2524608000000000000), // 2050-01-01
            tw_end: Timestamp::from_nanos(2556144000000000000),   // 2051-01-01
            batch_metadata: batch_metadata.clone(),
            metadata_json: None,
        });
        app.execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[])
            .unwrap();

        // Test 4: Microsecond precision
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: "4444444444444444444444444444444444444444444444444444444444444444".to_string(),
            tw_start: Timestamp::from_nanos(1704067200000001000), // +1 microsecond
            tw_end: Timestamp::from_nanos(1704067200000002000),   // +2 microseconds
            batch_metadata,
            metadata_json: None,
        });
        app.execute_contract(Addr::unchecked(USER), contract_addr, &store_msg, &[])
            .unwrap();
    }

    #[test]
    fn test_time_window_reversed_allowed() {
        // Note: Current implementation does NOT validate tw_end > tw_start
        // This is intentional to allow flexibility in batch ordering
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());
        let instantiate_msg = default_instantiate_msg();
        let contract_addr = app
            .instantiate_contract(contract_id, Addr::unchecked(ADMIN), &instantiate_msg, &[], "DeTrack", None)
            .unwrap();

        // Register node
        let register_msg = ExecuteMsg::Node(NodeExecuteMsg::RegisterNode {});
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &register_msg,
            &coins(instantiate_msg.deposit_tier1.u128(), NATIVE_DENOM),
        )
        .unwrap();

        let batch_metadata = vec![BatchInfo {
            batch_id: "batch-001".to_string(),
            gateway_did: r"did:c4e:gateway:test-gw1".to_string(),
            device_count: 5,
            snapshot_count: 10,
            batch_merkle_root: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        }];

        // tw_end < tw_start (reversed) - Currently ALLOWED
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: DATA_HASH.to_string(),
            tw_start: Timestamp::from_nanos(1704153600000000000),
            tw_end: Timestamp::from_nanos(1704067200000000000), // BEFORE start
            batch_metadata,
            metadata_json: None,
        });

        // This should succeed (no validation for tw_end > tw_start)
        let result = app.execute_contract(Addr::unchecked(USER), contract_addr, &store_msg, &[]);
        assert!(result.is_ok(), "Reversed time window should be allowed");
    }

    // =========================================================================
    // P0: DID FORMAT VALIDATION TESTS
    // =========================================================================

    #[test]
    fn test_did_format_validation_comprehensive() {
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());
        let instantiate_msg = default_instantiate_msg();
        let contract_addr = app
            .instantiate_contract(contract_id, Addr::unchecked(ADMIN), &instantiate_msg, &[], "DeTrack", None)
            .unwrap();

        // Register node
        let register_msg = ExecuteMsg::Node(NodeExecuteMsg::RegisterNode {});
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &register_msg,
            &coins(instantiate_msg.deposit_tier1.u128(), NATIVE_DENOM),
        )
        .unwrap();

        let batch_metadata = vec![BatchInfo {
            batch_id: "batch-001".to_string(),
            gateway_did: r"did:c4e:gateway:test-gw1".to_string(),
            device_count: 5,
            snapshot_count: 10,
            batch_merkle_root: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        }];

        // Test 1: Empty worker_did
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: "".to_string(),
            data_hash: "1111111111111111111111111111111111111111111111111111111111111111".to_string(),
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata: batch_metadata.clone(),
            metadata_json: None,
        });
        let err = app.execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[]).unwrap_err();
        assert!(matches!(err.downcast_ref::<ContractError>().unwrap(), ContractError::InvalidDidFormat { .. }));

        // Test 2: Wrong DID method (not "did:c4e")
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: "did:eth:worker:test".to_string(),
            data_hash: "2222222222222222222222222222222222222222222222222222222222222222".to_string(),
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata: batch_metadata.clone(),
            metadata_json: None,
        });
        let err = app.execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[]).unwrap_err();
        assert!(matches!(err.downcast_ref::<ContractError>().unwrap(), ContractError::InvalidDidFormat { .. }));

        // Test 3: Wrong type (gateway instead of worker)
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:gateway:wrongtype".to_string(),
            data_hash: "3333333333333333333333333333333333333333333333333333333333333333".to_string(),
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata: batch_metadata.clone(),
            metadata_json: None,
        });
        let err = app.execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[]).unwrap_err();
        assert!(matches!(err.downcast_ref::<ContractError>().unwrap(), ContractError::InvalidDidFormat { .. }));

        // Test 4: Invalid gateway_did format
        let invalid_batch = vec![BatchInfo {
            batch_id: "batch-001".to_string(),
            gateway_did: "did:c4e:worker:wrongtype".to_string(), // Should be gateway
            device_count: 5,
            snapshot_count: 10,
            batch_merkle_root: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        }];
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: "4444444444444444444444444444444444444444444444444444444444444444".to_string(),
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata: invalid_batch,
            metadata_json: None,
        });
        let err = app.execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[]).unwrap_err();
        assert!(matches!(err.downcast_ref::<ContractError>().unwrap(), ContractError::InvalidDidFormat { .. }));

        // Test 5: Missing colon separators
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: "did_c4e_worker_test".to_string(),
            data_hash: "5555555555555555555555555555555555555555555555555555555555555555".to_string(),
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata,
            metadata_json: None,
        });
        let err = app.execute_contract(Addr::unchecked(USER), contract_addr, &store_msg, &[]).unwrap_err();
        assert!(matches!(err.downcast_ref::<ContractError>().unwrap(), ContractError::InvalidDidFormat { .. }));
    }

    // =========================================================================
    // P1: BATCH BOUNDARY TESTS (Extended)
    // =========================================================================

    #[test]
    fn test_batch_boundary_exactly_100() {
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());
        let instantiate_msg = default_instantiate_msg();
        let contract_addr = app
            .instantiate_contract(contract_id, Addr::unchecked(ADMIN), &instantiate_msg, &[], "DeTrack", None)
            .unwrap();

        // Register node
        let register_msg = ExecuteMsg::Node(NodeExecuteMsg::RegisterNode {});
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &register_msg,
            &coins(instantiate_msg.deposit_tier1.u128(), NATIVE_DENOM),
        )
        .unwrap();

        // Create exactly 100 batches (boundary test)
        let batch_metadata: Vec<BatchInfo> = (0..100)
            .map(|i| BatchInfo {
                batch_id: format!("batch-{:03}", i),
                gateway_did: format!("did:c4e:gateway:gw{}", i % 5),
                device_count: 5,
                snapshot_count: 10,
                batch_merkle_root: format!("{:0<64}", format!("{:x}", i)),
            })
            .collect();

        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: DATA_HASH.to_string(),
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata,
            metadata_json: None,
        });

        // Should succeed with exactly 100 batches
        let res = app.execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[]).unwrap();
        
        let store_proof_event = res.events.iter().find(|e| e.ty == "wasm-store_proof").unwrap();
        assert_eq!(
            store_proof_event.attributes.iter().find(|a| a.key == "batch_count").unwrap().value,
            "100"
        );

        // Verify proof stored correctly
        let query_msg = QueryMsg::Proof { id: 0 };
        let proof: ProofResponse = app.wrap().query_wasm_smart(contract_addr, &query_msg).unwrap();
        assert_eq!(proof.batch_metadata.len(), 100);
    }

    #[test]
    fn test_batch_single_vs_multiple() {
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());
        let instantiate_msg = default_instantiate_msg();
        let contract_addr = app
            .instantiate_contract(contract_id, Addr::unchecked(ADMIN), &instantiate_msg, &[], "DeTrack", None)
            .unwrap();

        // Register node
        let register_msg = ExecuteMsg::Node(NodeExecuteMsg::RegisterNode {});
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &register_msg,
            &coins(instantiate_msg.deposit_tier1.u128(), NATIVE_DENOM),
        )
        .unwrap();

        // Test 1: Single batch
        let batch_metadata = vec![BatchInfo {
            batch_id: "batch-single".to_string(),
            gateway_did: r"did:c4e:gateway:test-gw1".to_string(),
            device_count: 100,
            snapshot_count: 500,
            batch_merkle_root: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        }];

        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: "1111111111111111111111111111111111111111111111111111111111111111".to_string(),
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata,
            metadata_json: None,
        });

        app.execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[]).unwrap();

        // Test 2: Multiple batches from same gateway
        let batch_metadata = vec![
            BatchInfo {
                batch_id: "batch-001".to_string(),
                gateway_did: r"did:c4e:gateway:test-gw2".to_string(),
                device_count: 10,
                snapshot_count: 50,
                batch_merkle_root: "1111111111111111111111111111111111111111111111111111111111111111".to_string(),
            },
            BatchInfo {
                batch_id: "batch-002".to_string(),
                gateway_did: r"did:c4e:gateway:test-gw2".to_string(),
                device_count: 10,
                snapshot_count: 50,
                batch_merkle_root: "2222222222222222222222222222222222222222222222222222222222222222".to_string(),
            },
        ];

        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: "2222222222222222222222222222222222222222222222222222222222222222".to_string(),
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata,
            metadata_json: None,
        });

        app.execute_contract(Addr::unchecked(USER), contract_addr, &store_msg, &[]).unwrap();
    }

    // =========================================================================
    // P2: QUERY TESTS WITH TIMESTAMP ORDERING
    // =========================================================================

    #[test]
    fn test_query_proofs_with_timestamp_ordering() {
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());
        let instantiate_msg = default_instantiate_msg();
        let contract_addr = app
            .instantiate_contract(contract_id, Addr::unchecked(ADMIN), &instantiate_msg, &[], "DeTrack", None)
            .unwrap();

        // Register node
        let register_msg = ExecuteMsg::Node(NodeExecuteMsg::RegisterNode {});
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &register_msg,
            &coins(instantiate_msg.deposit_tier1.u128(), NATIVE_DENOM),
        )
        .unwrap();

        let batch_metadata = vec![BatchInfo {
            batch_id: "batch-001".to_string(),
            gateway_did: r"did:c4e:gateway:test-gw1".to_string(),
            device_count: 5,
            snapshot_count: 10,
            batch_merkle_root: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        }];

        // Store 3 proofs with different timestamps
        // Proof 1: Jan 1, 2024
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: "1111111111111111111111111111111111111111111111111111111111111111".to_string(),
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata: batch_metadata.clone(),
            metadata_json: None,
        });
        app.execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[]).unwrap();

        // Proof 2: Feb 1, 2024
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: "2222222222222222222222222222222222222222222222222222222222222222".to_string(),
            tw_start: Timestamp::from_nanos(1706745600000000000),
            tw_end: Timestamp::from_nanos(1706832000000000000),
            batch_metadata: batch_metadata.clone(),
            metadata_json: None,
        });
        app.execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[]).unwrap();

        // Proof 3: Mar 1, 2024
        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: "3333333333333333333333333333333333333333333333333333333333333333".to_string(),
            tw_start: Timestamp::from_nanos(1709251200000000000),
            tw_end: Timestamp::from_nanos(1709337600000000000),
            batch_metadata,
            metadata_json: None,
        });
        app.execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[]).unwrap();

        // Query all proofs (ordered by ID, not timestamp)
        let query_msg = QueryMsg::Proofs {
            start_after: None,
            limit: None,
        };
        let proofs: ProofsResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &query_msg).unwrap();
        assert_eq!(proofs.proofs.len(), 3);

        // Verify chronological order (by ID)
        assert_eq!(proofs.proofs[0].id, 0);
        assert_eq!(proofs.proofs[1].id, 1);
        assert_eq!(proofs.proofs[2].id, 2);

        // Verify timestamps are preserved correctly
        assert_eq!(proofs.proofs[0].tw_start, Timestamp::from_nanos(1704067200000000000));
        assert_eq!(proofs.proofs[1].tw_start, Timestamp::from_nanos(1706745600000000000));
        assert_eq!(proofs.proofs[2].tw_start, Timestamp::from_nanos(1709251200000000000));

        // Test pagination
        let query_msg = QueryMsg::Proofs {
            start_after: Some(0),
            limit: Some(2),
        };
        let proofs: ProofsResponse = app.wrap().query_wasm_smart(contract_addr, &query_msg).unwrap();
        assert_eq!(proofs.proofs.len(), 2);
        assert_eq!(proofs.proofs[0].id, 1);
        assert_eq!(proofs.proofs[1].id, 2);
    }

    #[test]
    fn test_query_by_worker_and_gateway_with_timestamps() {
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());
        let instantiate_msg = default_instantiate_msg();
        let contract_addr = app
            .instantiate_contract(contract_id, Addr::unchecked(ADMIN), &instantiate_msg, &[], "DeTrack", None)
            .unwrap();

        // Register node
        let register_msg = ExecuteMsg::Node(NodeExecuteMsg::RegisterNode {});
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &register_msg,
            &coins(instantiate_msg.deposit_tier1.u128(), NATIVE_DENOM),
        )
        .unwrap();

        // Store 2 proofs from same worker with different gateways
        let batch_metadata1 = vec![BatchInfo {
            batch_id: "batch-001".to_string(),
            gateway_did: r"did:c4e:gateway:test-gw1".to_string(),
            device_count: 5,
            snapshot_count: 10,
            batch_merkle_root: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        }];

        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: "1111111111111111111111111111111111111111111111111111111111111111".to_string(),
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata: batch_metadata1,
            metadata_json: None,
        });
        app.execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[]).unwrap();

        let batch_metadata2 = vec![BatchInfo {
            batch_id: "batch-002".to_string(),
            gateway_did: r"did:c4e:gateway:test-gw2".to_string(),
            device_count: 3,
            snapshot_count: 8,
            batch_merkle_root: "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210".to_string(),
        }];

        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            data_hash: "2222222222222222222222222222222222222222222222222222222222222222".to_string(),
            tw_start: Timestamp::from_nanos(1706745600000000000),
            tw_end: Timestamp::from_nanos(1706832000000000000),
            batch_metadata: batch_metadata2,
            metadata_json: None,
        });
        app.execute_contract(Addr::unchecked(USER), contract_addr.clone(), &store_msg, &[]).unwrap();

        // Query by worker - should return both proofs
        let query_msg = QueryMsg::ProofsByWorker {
            worker_did: r"did:c4e:worker:detrack1".to_string(),
            start_after: None,
            limit: None,
        };
        let proofs: ProofsResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &query_msg).unwrap();
        assert_eq!(proofs.proofs.len(), 2);

        // Query by gateway1 - should return only first proof
        let query_msg = QueryMsg::ProofsByGateway {
            gateway_did: r"did:c4e:gateway:test-gw1".to_string(),
            start_after: None,
            limit: None,
        };
        let proofs: ProofsResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &query_msg).unwrap();
        assert_eq!(proofs.proofs.len(), 1);
        assert_eq!(proofs.proofs[0].tw_start, Timestamp::from_nanos(1704067200000000000));

        // Query by gateway2 - should return only second proof
        let query_msg = QueryMsg::ProofsByGateway {
            gateway_did: r"did:c4e:gateway:test-gw2".to_string(),
            start_after: None,
            limit: None,
        };
        let proofs: ProofsResponse = app.wrap().query_wasm_smart(contract_addr, &query_msg).unwrap();
        assert_eq!(proofs.proofs.len(), 1);
        assert_eq!(proofs.proofs[0].tw_start, Timestamp::from_nanos(1706745600000000000));
    }

    // =========================================================================
    // REAL DID CONTRACT INTEGRATION TEST (requires real DID contract deployed)
    // =========================================================================

    #[test]
    fn test_real_did_contract_address_configured() {
        // This test verifies that the real DID contract address can be configured
        // Note: Actual DID verification is mocked in #[cfg(test)] mode
        let mut app = mock_app();
        let contract_id = app.store_code(detrack_contract());
        
        // Use REAL DID contract address
        let mut instantiate_msg = default_instantiate_msg();
        instantiate_msg.did_contract_address = "c4e14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s86dt7n".to_string();
        
        let contract_addr = app
            .instantiate_contract(contract_id, Addr::unchecked(ADMIN), &instantiate_msg, &[], "DeTrack", None)
            .unwrap();

        // Verify DID contract address is stored correctly
        let query_msg = QueryMsg::Config {};
        let config: ConfigResponse = app.wrap().query_wasm_smart(contract_addr.clone(), &query_msg).unwrap();
        assert_eq!(config.did_contract_address, "c4e14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s86dt7n");

        // Register node with real DID contract address
        let register_msg = ExecuteMsg::Node(NodeExecuteMsg::RegisterNode {});
        app.execute_contract(
            Addr::unchecked(USER),
            contract_addr.clone(),
            &register_msg,
            &coins(instantiate_msg.deposit_tier1.u128(), NATIVE_DENOM),
        )
        .unwrap();

        // Store proof (DID verification is mocked in test mode, but address is real)
        let batch_metadata = vec![BatchInfo {
            batch_id: "batch-001".to_string(),
            gateway_did: r"did:c4e:gateway:test-gw1".to_string(),
            device_count: 5,
            snapshot_count: 10,
            batch_merkle_root: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        }];

        let store_msg = ExecuteMsg::Node(NodeExecuteMsg::StoreProof {
            worker_did: r"did:c4e:worker:detrack2".to_string(),
            data_hash: DATA_HASH.to_string(),
            tw_start: Timestamp::from_nanos(1704067200000000000),
            tw_end: Timestamp::from_nanos(1704153600000000000),
            batch_metadata,
            metadata_json: Some(r#"{"note": "Using real DID contract address"}"#.to_string()),
        });

        let res = app.execute_contract(Addr::unchecked(USER), contract_addr, &store_msg, &[]).unwrap();
        
        // Verify event emitted
        let store_proof_event = res.events.iter().find(|e| e.ty == "wasm-store_proof").unwrap();
        assert_eq!(
            store_proof_event.attributes.iter().find(|a| a.key == "worker_did").unwrap().value,
            r"did:c4e:worker:detrack2"
        );
    }
}
