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
| F001 | Project Initialization and Main Structure | ✅     | High     | main   | -                 | 3.23%    | Implemented          | -               | Main directory and configuration files completed. Unit test covers struct, integration test present but coverage limited by tarpaulin/anchor compatibility. |
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

---

_This file is maintained automatically as part of the development workflow._ 