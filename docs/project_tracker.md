# Project Development Tracker

## Status Legend

* 🔜 - Planned (Ready for development)
* 🚧 - In Progress (Currently being developed)
* ✅ - Completed
* 🧪 - In Testing
* 🐛 - Has known issues

## Test Status Legend

* ✓ - Tests Passed
* ✗ - Tests Failed
* ⏳ - Tests In Progress
* ⚠️  - Tests Blocked
* * * Not Started

## Feature Tasks

| ID   | Feature Name                | Status | Priority | Branch | Assigned To (MAC) | Coverage | Unit Tests | Regression Tests | Notes                  |
| ---- | -------------------------- | ------ | -------- | ------ | ----------------- | -------- | ---------- | ---------------- | ---------------------- |
| F001 | Project Initialization and Main Structure | ✅     | High     | main   | -                 | 66.12%    | Implemented          | -               | Main directory and configuration files completed. Unit test covers struct, integration test present but coverage limited by tarpaulin/anchor compatibility. |
| F002 | Oracle Node Registration and Management   | ✅     | High     | feature/support-cross-chain      | c6:c4:e5:e8:c6:4b                 | >80%        | Implemented          | Implemented               | All integration tests and main flows completed |
| F003 | Cross-chain Request and Message Handling  | ✅     | High     | feature/cross-chain-request      | c6:c4:e5:e8:c6:4b                 | >80%        | Implemented          | Implemented               | send_request only checks whitelist, transmit/receive_message checks threshold signature.|
| F004 | Whitelist and Chain Support Configuration | ✅     | Medium   | feature/whitelist-chain-support      | c6:c4:e5:e8:c6:4b                 | >80%        | Implemented          | Implemented               | Roles refactored: owner (admin), sender (cross-chain initiator), transmitter (cross-chain submitter), signer (off-chain signature verifier). Multisig accounts and InitMultisig removed. send_request only checks whitelist, transmit/receive_message checks threshold signature.|

## Technical Debt & Refactoring

| ID   | Task Description           | Status | Priority | Branch | Assigned To (MAC) | Unit Tests | Regression Tests | Notes               |
| ---- | --------------------------| ------ | -------- | ------ | ----------------- | ---------- | ---------------- | ------------------- |
| T001 | Optimize Contract Data Structure          | 🔜     | Medium   | -      | -                 | -        | -          | -               | Improve performance and scalability   |

## Bug Fixes

| ID   | Bug Description           | Status | Priority | Branch | Assigned To (MAC) | Unit Tests | Regression Tests | Notes                 |
| ---- | ------------------------- | ------ | -------- | ------ | ----------------- | ---------- | ---------------- | --------------------- |
| B001 | -                         | -      | -        | -      | -                 | -          | -               | -                     |

## Development Metrics

* Total Test Coverage: >80%
* Last Updated: 2025-05-06

## Upcoming Automated Tasks

| ID   | Task Description             | Dependency | Estimated Completion  |
| ---- | ---------------------------- | ---------- | --------------------- |
| A001 | Generate node account and registration instruction skeleton | F002       | After F002 start      |
| A002 | Auto-generate integration test scripts    | F005       | After F005 start      |

## Notes & Action Items

* Initial project structure completed
* Node account, instructions, and test cases to be designed
* CI/CD pipeline configuration to be supplemented
* Documentation needs to be continuously improved as features progress

## Test Tracker

### 已完成测试（programs/oracle/tests/oracle.rs）

| 测试点 | 功能点 | 类型 | 状态 | 备注 |
| ------ | ------ | ---- | ---- | ---- |
| test_set_chain_whitelist_logic | 链白名单设置与权限校验 | 单元 | ✓ | admin与非admin分支覆盖 |
| test_set_oracle_nodes_logic | 节点设置与权限校验 | 单元 | ✓ | admin与非admin分支覆盖 |
| test_add_sender_logic | sender添加与边界 | 单元 | ✓ | 重复添加/满员/非admin分支 |
| test_remove_sender_logic | sender移除与边界 | 单元 | ✓ | 不存在/非admin分支覆盖 |
| test_send_request_logic | 跨链请求发送与白名单校验 | 单元 | ✓ | sender/chain_id白名单分支 |
| test_verify_signature_secp256k1_always_false | secp256k1签名校验（单） | 单元 | ✓ | 本地无secp256k1指令，预期false |
| test_verify_signatures_secp256k1_always_false | secp256k1签名校验（批） | 单元 | ✓ | 本地无secp256k1指令，预期false |
| test_initialize_logic_success | 初始化流程（成功） | 单元 | ✓ | admin/白名单/节点初始化 |
| test_initialize_logic_fail_already_initialized | 初始化流程（已初始化异常） | 单元 | ✓ | 已初始化分支覆盖 |
| test_set_admin_logic_success | admin切换（成功） | 单元 | ✓ | 权限正确分支 |
| test_set_admin_logic_fail_not_admin | admin切换（非admin异常） | 单元 | ✓ | 非admin分支覆盖 |
| test_transmit_multisig_threshold | transmit多签门限与边界 | 单元 | ✓ | 签名数/门限/账户mock覆盖 |
| test_transmit_signature_below_threshold | transmit签名不足 | 单元 | ✓ | 门限不足分支 |
| test_transmit_with_duplicate_signatures | transmit重复签名 | 单元 | ✓ | 签名去重逻辑 |
| test_transmit_with_too_many_signatures | transmit签名超限 | 单元 | ✓ | 超过节点数分支 |

### 在做/计划测试

| 测试点 | 功能点 | 类型 | 状态 | 备注 |
| ------ | ------ | ---- | ---- | ---- |
| 集成测试（Anchor/CI自动化） | 合约端到端流程 | 集成 | ⏳ | 需Anchor/本地链环境完善 |
| 节点注册/更新指令集成 | register/update_node指令 | 集成 | 🔜 | 需补充指令与mock账户 |
| 跨链消息完整流转 | send/receive_message指令 | 集成 | 🔜 | 需模拟链间消息与状态流转 |

### 未来建议测试

| 测试点 | 功能点 | 类型 | 状态 | 备注 |
| ------ | ------ | ---- | ---- | ---- |
| 异常路径与安全性 | 非法签名、无效链ID、节点禁用等 | 集成/安全 | * | 需补充异常与攻击场景 |
| 性能与压力测试 | 大量消息/节点/签名 | 性能 | * | 评估系统极限与瓶颈 |
| 动态白名单变更 | 运行时变更sender/chain | 集成 | * | 检查热更新与一致性 |
| 多链扩展性 | 新链接入与兼容性 | 集成 | * | 新增chain_id与消息类型 |

## Ramp Test Tracker (English)

### Completed Tests (programs/ramp/tests/ramp.rs)

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

### In Progress / Planned Tests

| Test Name | Feature/Scenario | Type | Status | Notes |
| --------- | --------------- | ---- | ------ | ----- |
| test_transmit_real_secp256k1 | Real secp256k1 signature (integration) | Integration | ⏳ | Async, requires Anchor/CI environment |
| Integration tests (CPI, cross-contract) | End-to-end contract flows | Integration | 🔜 | Placeholder in mod.rs, not yet implemented |

### Suggested Future Tests

| Test Name | Feature/Scenario | Type | Status | Notes |
| --------- | --------------- | ---- | ------ | ----- |
| Error path and security | Invalid signatures, unauthorized access, node disable | Integration/Security | * | More attack/edge cases needed |
| Performance and stress | High volume messages/nodes/signatures | Performance | * | Evaluate system limits |
| Dynamic whitelist changes | Runtime sender/chain updates | Integration | * | Test hot updates and consistency |
| Multi-chain extensibility | New chain onboarding, compatibility | Integration | * | Add new chain_id and message types |

---

_This file is maintained automatically as part of the development workflow._ 