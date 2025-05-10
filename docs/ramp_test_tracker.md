# Ramp Test Tracker

## Completed Tests (programs/ramp/tests/ramp.rs & integration)

| Test Name | Feature/Scenario | Type | Status | Notes |
| --------- | --------------- | ---- | ------ | ----- |
| test_initialize | Config initialization | Unit | ✓ | Covers authority, config_data, is_initialized |
| test_set_authority | Authority transfer | Unit | ✓ | Authority can be changed |
| test_set_oracle_nodes | Oracle node management | Unit | ✓ | Set new oracle nodes |
| test_initialize_already_initialized | Prevent re-initialization | Unit | ✓ | Error if already initialized |
| test_set_oracle_nodes_unauthorized | Unauthorized node set | Unit | ✓ | Only authority can set nodes |
| test_add_sender | Add sender to whitelist | Unit | ✓ | Normal, duplicate, full whitelist cases |
| test_remove_sender | Remove sender from whitelist | Unit | ✓ | Normal and non-existent sender |
| test_set_chain_whitelist | Set source/destination chain whitelist | Unit | ✓ | Authority and unauthorized cases |
| test_add_sender_unauthorized | Unauthorized add sender | Unit | ✓ | Only authority can add |
| test_remove_sender_unauthorized | Unauthorized remove sender | Unit | ✓ | Only authority can remove |
| test_transmit_threshold | Multisig threshold logic | Unit | ✓ | Succeeds if enough unique validators |
| test_transmit_insufficient_signatures | Insufficient signatures | Unit | ✓ | Fails if below threshold |
| test_transmit_non_whitelisted_node | Non-whitelisted node signature | Unit | ✓ | Fails if signer not in oracle_nodes |
| test_transmit_duplicate_signatures | Duplicate signatures | Unit | ✓ | Only unique counted |
| test_transmit_data_tampered | Data tampering | Unit | ✓ | Fails if data is tampered |
| test_send_request_success | Send request (happy path) | Unit | ✓ | Message ID hash, whitelist, metadata |
| test_send_request_sender_not_whitelisted | Sender not whitelisted | Unit | ✓ | Fails if sender not in whitelist |
| test_send_request_chain_not_whitelisted | Chain not whitelisted | Unit | ✓ | Fails if destination chain not in whitelist |
| test_send_request_message_id_consistency | Message ID consistency | Unit | ✓ | Ensures deterministic message ID |
| test_transmit_real_secp256k1 | Real secp256k1 signature (integration) | Integration | ✓ | Uses real cryptographic signatures, on-chain secp256k1 verification |
| test_initialize_idempotent | Idempotent initialization | Integration | ✓ | Safe to re-run, cleans up state |
| test_set_admin | Set admin (integration) | Integration | ✓ | Authority transfer, error on unauthorized |
| test_set_oracle_nodes_integration | Oracle node management | Integration | ✓ | Add/remove nodes, unauthorized cases |
| test_add_sender_integration | Add sender (integration) | Integration | ✓ | Normal, duplicate, full whitelist, unauthorized |
| test_remove_sender_integration | Remove sender (integration) | Integration | ✓ | Normal, not found, unauthorized |
| test_set_chain_whitelist_integration | Chain whitelist (integration) | Integration | ✓ | Authority and unauthorized cases |
| test_send_request_integration | Send request (integration) | Integration | ✓ | Sender/chain not whitelisted, message ID, metadata |
| test_transmit_integration | Transmit (integration) | Integration | ✓ | Valid signatures, insufficient signatures, non-whitelisted node, duplicate signatures, data tampering, all error/security paths |

## Future/Planned Tests

| Test Name | Feature/Scenario | Type | Status | Notes |
| --------- | --------------- | ---- | ------ | ----- |
| Performance and stress | High volume messages/nodes/signatures | Performance | * | Evaluate system limits |
| Dynamic whitelist changes | Runtime sender/chain updates | Integration | * | Test hot updates and consistency |
| Multi-chain extensibility | New chain onboarding, compatibility | Integration | * | Add new chain_id and message types |

All major instructions and error/security paths are now fully covered by both unit and integration tests. Integration tests use real cryptographic signatures and exercise all on-chain logic. All tests are idempotent, robust, and pass in CI and local environments.