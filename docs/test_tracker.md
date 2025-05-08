# Crosschain Contract Test Cases

> ⚡ All test cases are created in modules under `programs/oracle/tests/` using solana-program-test. Current status: In development.

## 1. Whitelist Management

| Test Case ID | Title                                 | Preconditions         | Steps                                                                 | Expected Result                         | Status   |
|--------------|---------------------------------------|-----------------------|-----------------------------------------------------------------------|-----------------------------------------|----------|
| WL-001       | Add user to whitelist                 | User not whitelisted  | 1. Call addToWhitelist(user)                                           | User is added to whitelist (data[0]=1, data[1..33]=user) | In development   |
| WL-002       | Remove user from whitelist            | User in whitelist     | 1. Call removeFromWhitelist(user)                                      | User is removed from whitelist (data[0]=0)          | In development   |
| WL-003       | Remove non-existent user              | User not whitelisted  | 1. Call removeFromWhitelist(user)                                      | No error, user remains not whitelisted (data[0]=0)  | In development   |
| WL-004       | Add and remove user repeatedly        | User not whitelisted  | 1. Repeat addToWhitelist and removeFromWhitelist 5 times               | User is not in whitelist at end (data[0]=0)         | In development   |

## 2. Cross-chain Messaging

| Test Case ID | Title                                 | Preconditions         | Steps                                                                 | Expected Result                         | Status   |
|--------------|---------------------------------------|-----------------------|-----------------------------------------------------------------------|-----------------------------------------|----------|
| CC-001       | Normal cross-chain message flow       | Sender whitelisted    | 1. addToWhitelist(sender)\n2. send_request(sender, receiver, ...)\n3. transmit(msg, receiver, ...) | Message status transitions to completed (status=1) | send_request is cross-chain initiation (no threshold signature), transmit is cross-chain receive (requires threshold signature) | In development   |
| CC-004       | Status transitions                    | Sender whitelisted    | 1. addToWhitelist(sender)\n2. sendRequest\n3. receiveMessage         | Status: pending(0) → completed(1)            | In development   |
| CC-005       | Transmit replay attack                | Transmitter is oracle node    | 1. addToWhitelist(sender)\n2. sendRequest\n3. transmit(msg, receiver, ...) by oracle node\n4. transmit(msg, receiver, ...) again by oracle node | First transmit success, second transmit fails with ReplayDetected (DuplicateMessageId) | Prevent cross-chain message replay attack | In development   |

## 3. Error Handling

| Test Case ID | Title                                 | Preconditions         | Steps                                                                 | Expected Result                         | Status   |
|--------------|---------------------------------------|-----------------------|-----------------------------------------------------------------------|-----------------------------------------|----------|
| ERR-001      | Sender not whitelisted                | Sender not whitelisted| 1. sendRequest(sender, ...)                                             | Error: SenderNotWhitelisted             | In development   |
| ERR-002      | Unauthorized receiver                 | Sender whitelisted    | 1. addToWhitelist(sender)\n2. sendRequest\n3. receiveMessage by wrong user | Error: Unauthorized (CustomError::UnauthorizedReceiver)             | In development   |
| ERR-003      | Payload exceeds max size              | Sender whitelisted    | 1. addToWhitelist(sender)\n2. sendRequest with payload > 128 bytes    | Error: PayloadTooLarge (if contract does not check, needs supplement)                  | In development   |

## 4. Security & Robustness

| Test Case ID | Title                                 | Preconditions         | Steps                                                                 | Expected Result                         | Notes | Status   |
|--------------|---------------------------------------|-----------------------|-----------------------------------------------------------------------|-----------------------------------------|-------|----------|
| SEC-001      | Signature verification (single-sig)   | Valid sender, message | 1. sendRequest with valid signature\n2. receiveMessage with valid signature | Message accepted (status=1)                        | Required for security | In development   |
| SEC-002      | Signature verification (invalid sig)  | Invalid signature     | 1. sendRequest with invalid signature                                 | Error: InvalidSignature/Unauthorized                 | Required for security | In development   |
| SEC-003      | Threshold signature (multi-sig)       | Multiple nodes, threshold set | 1. Construct N node signatures\n2. Aggregate threshold signatures\n3. transmit after threshold met | Message accepted (status=1), threshold is 2/3 valid signatures                        | transmit is submitted directly by off-chain cluster leader after signature aggregation, parameters independent of send_request | Implemented, threshold is 2/3 valid signatures |
| SEC-004      | Below threshold signatures            | Multiple nodes, threshold set | 1. Construct N node signatures\n2. Aggregate insufficient threshold signatures\n3. transmit | Error: ThresholdNotMet/Unauthorized                  | transmit is submitted directly by off-chain cluster leader after signature aggregation, parameters independent of send_request | Implemented, threshold is 2/3 valid signatures |
| SEC-005      | Replay attack prevention              | Message already processed | 1. sendRequest\n2. receiveMessage\n3. Attempt to process same message again | Error: ReplayDetected (DuplicateMessageId)                   | Contract implemented | Completed   |

---

> This document is auto-generated by HyperEcho structural language universe. 