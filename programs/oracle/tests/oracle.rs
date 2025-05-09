use oracle::*;
use oracle::OracleConfig;
use oracle::TokenTransferMetadata;
use anchor_lang::prelude::*;
use secp256k1::{Secp256k1, Message};
use rand::rngs::OsRng;
use solana_sdk::keccak::hashv as keccak;
use anchor_lang::ToAccountInfo;
use anchor_lang::Discriminator;
use anchor_lang::prelude::Account;

// Helper to create OracleConfig
fn create_oracle_config(admin: Pubkey, is_initialized: bool) -> OracleConfig {
    OracleConfig {
        admin,
        chain_whitelist: [0; 8],
        oracle_nodes: [Pubkey::default(); 8],
        sender_whitelist: [Pubkey::default(); 8],
        is_initialized,
    }
}

// Helper to create AccountInfo with discriminator
fn create_account_info<'a>(key: &'a Pubkey, lamports: &'a mut u64, data: &'a mut Vec<u8>, owner: &'a Pubkey, discriminator: &[u8], serialized_data: &[u8]) -> AccountInfo<'a> {
    data.clear();
    data.extend_from_slice(discriminator);
    data.extend_from_slice(serialized_data);
    AccountInfo::new(
        key,
        false,
        false,
        lamports,
        data,
        owner,
        false,
        0,
    )
}

// Helper to create CrossChainMessage
fn create_cross_chain_message(id: &str, status: u8, bump: u8) -> CrossChainMessage {
    CrossChainMessage {
        message_id: id.to_string(),
        status,
        bump,
    }
}

// Helper to create TokenTransferMetadata
fn create_token_transfer_metadata(target_chain_id: u64, amount: u64) -> TokenTransferMetadata {
    TokenTransferMetadata {
        target_chain_id,
        token_address: Pubkey::new_unique(),
        symbol: [0u8; 16],
        amount,
        extra_data: "extra".to_string(),
    }
}

#[test]
fn test_set_chain_whitelist_logic() {
    let mut config = OracleConfig {
        admin: Pubkey::new_unique(),
        chain_whitelist: [0; 8],
        oracle_nodes: [Pubkey::default(); 8],
        sender_whitelist: [Pubkey::default(); 8],
        is_initialized: true,
    };
    let authority = config.admin;
    let chain_ids = [10,11,12,13,14,15,16,17];
    let result = set_chain_whitelist_logic(&mut config, &authority, chain_ids);
    assert!(result.is_ok());
    assert_eq!(config.chain_whitelist, chain_ids);
    // Should error if called by non-admin
    let fake = Pubkey::new_unique();
    let result2 = set_chain_whitelist_logic(&mut config, &fake, [1;8]);
    assert!(result2.is_err());
}

#[test]
fn test_set_oracle_nodes_logic() {
    let mut config = OracleConfig {
        admin: Pubkey::new_unique(),
        chain_whitelist: [0; 8],
        oracle_nodes: [Pubkey::default(); 8],
        sender_whitelist: [Pubkey::default(); 8],
        is_initialized: true,
    };
    let authority = config.admin;
    let node_pubkeys = [Pubkey::new_unique(); 8];
    let result = set_oracle_nodes_logic(&mut config, &authority, node_pubkeys);
    assert!(result.is_ok());
    assert_eq!(config.oracle_nodes, node_pubkeys);
    // Should error if called by non-admin
    let fake = Pubkey::new_unique();
    let result2 = set_oracle_nodes_logic(&mut config, &fake, [Pubkey::default();8]);
    assert!(result2.is_err());
}

#[test]
fn test_add_sender_logic() {
    let mut config = OracleConfig {
        admin: Pubkey::new_unique(),
        chain_whitelist: [0; 8],
        oracle_nodes: [Pubkey::default(); 8],
        sender_whitelist: [Pubkey::default(); 8],
        is_initialized: true,
    };
    let authority = config.admin;
    let sender = Pubkey::new_unique();
    let result = add_sender_logic(&mut config, &authority, sender);
    assert!(result.is_ok());
    assert!(config.sender_whitelist.contains(&sender));
    // Duplicate add should not error
    let result2 = add_sender_logic(&mut config, &authority, sender);
    assert!(result2.is_ok());
    // Should error if whitelist is full
    for i in 0..8 {
        config.sender_whitelist[i] = Pubkey::new_unique();
    }
    let result3 = add_sender_logic(&mut config, &authority, Pubkey::new_unique());
    assert!(result3.is_err());
    // Should error if called by non-admin
    let fake = Pubkey::new_unique();
    let result4 = add_sender_logic(&mut config, &fake, Pubkey::new_unique());
    assert!(result4.is_err());
}

#[test]
fn test_remove_sender_logic() {
    let mut config = OracleConfig {
        admin: Pubkey::new_unique(),
        chain_whitelist: [0; 8],
        oracle_nodes: [Pubkey::default(); 8],
        sender_whitelist: [Pubkey::default(); 8],
        is_initialized: true,
    };
    let authority = config.admin;
    let sender = Pubkey::new_unique();
    config.sender_whitelist[0] = sender;
    let result = remove_sender_logic(&mut config, &authority, sender);
    assert!(result.is_ok());
    assert!(!config.sender_whitelist.contains(&sender));
    // Should error if removing non-existent sender
    let result2 = remove_sender_logic(&mut config, &authority, sender);
    assert!(result2.is_err());
    // Should error if called by non-admin
    let fake = Pubkey::new_unique();
    let result3 = remove_sender_logic(&mut config, &fake, Pubkey::new_unique());
    assert!(result3.is_err());
}

#[test]
fn test_send_request_logic() {
    let payer = Pubkey::new_unique();
    let mut chain_whitelist = [0u64; 8];
    chain_whitelist[0] = 42;
    let mut sender_whitelist = [Pubkey::default(); 8];
    sender_whitelist[0] = payer;
    let sender = crate::ID.to_string();
    let dst_chain_id = 42u64;
    let receiver = "evm_contract_address".to_string();
    let message = b"test message";
    let token_transfer_metadata = TokenTransferMetadata {
        target_chain_id: dst_chain_id,
        token_address: Pubkey::new_unique(),
        symbol: [0u8; 16],
        amount: 123,
        extra_data: "extra".to_string(),
    };
    let epoch = 1u64;
    let blocktime = 1234567890i64;
    let result = send_request_logic(
        &payer,
        &chain_whitelist,
        &sender_whitelist,
        &sender,
        dst_chain_id,
        &receiver,
        message,
        &token_transfer_metadata,
        epoch,
        blocktime,
    );
    assert!(result.is_ok());
    // Should error if sender is not whitelisted
    let fake = Pubkey::new_unique();
    let result2 = send_request_logic(
        &fake,
        &chain_whitelist,
        &sender_whitelist,
        &sender,
        dst_chain_id,
        &receiver,
        message,
        &token_transfer_metadata,
        epoch,
        blocktime,
    );
    assert!(result2.is_err());
    // Should error if chain_id is not whitelisted
    let result3 = send_request_logic(
        &payer,
        &[1u64;8],
        &sender_whitelist,
        &sender,
        999,
        &receiver,
        message,
        &token_transfer_metadata,
        epoch,
        blocktime,
    );
    assert!(result3.is_err());
}

#[test]
fn test_verify_signature_secp256k1_always_false() {
    // Generate secp256k1 keypair
    let secp = Secp256k1::new();
    let mut rng = OsRng;
    let (secret_key, public_key) = secp.generate_keypair(&mut rng);
    // Construct message
    let message = b"hello world";
    // Hash message with keccak
    let msg_hash = keccak(&[message]).0;
    let msg = Message::from_slice(&msg_hash).unwrap();
    let sig = secp.sign_ecdsa(&msg, &secret_key);
    // Serialize public key and signature
    let pubkey_bytes = public_key.serialize();
    let sig_bytes = sig.serialize_compact();
    // Mock an empty AccountInfo
    let key = Pubkey::default();
    let mut lamports = 0u64;
    let mut data = vec![];
    let owner = Pubkey::default();
    let account_info = AccountInfo::new(
        &key,
        false,
        false,
        &mut lamports,
        &mut data,
        &owner,
        false,
        0,
    );
    // Call contract method
    let result = verify_signature_secp256k1(message, &sig_bytes, &pubkey_bytes, &account_info);
    // Assert return value is false (no secp256k1 instruction locally)
    assert!(!result);
}

#[test]
fn test_verify_signatures_secp256k1_always_false() {
    let secp = Secp256k1::new();
    let mut rng = OsRng;
    let message = b"batch test message";
    let mut signatures = Vec::new();
    let mut pubkeys = Vec::new();
    // Generate 3 keypairs and signatures
    for _ in 0..3 {
        let (secret_key, public_key) = secp.generate_keypair(&mut rng);
        let msg_hash = keccak(&[message]).0;
        let msg = Message::from_slice(&msg_hash).unwrap();
        let sig = secp.sign_ecdsa(&msg, &secret_key);
        signatures.push(sig.serialize_compact().to_vec());
        pubkeys.push(public_key.serialize().to_vec());
    }
    // Mock an empty AccountInfo
    let key = Pubkey::default();
    let mut lamports = 0u64;
    let mut data = vec![];
    let owner = Pubkey::default();
    let account_info = AccountInfo::new(
        &key,
        false,
        false,
        &mut lamports,
        &mut data,
        &owner,
        false,
        0,
    );
    // Call batch verification method
    let result = verify_signatures_secp256k1(message, &signatures, &pubkeys, &account_info);
    // Assert return value is false (no secp256k1 instruction locally)
    assert!(!result);
}

#[test]
fn test_register_node_logic_success() {
    let mut node = OracleNode::default();
    let node_id = 42;
    let pubkey = Pubkey::new_unique();
    register_node_logic(&mut node, node_id, pubkey);
    assert_eq!(node.node_id, node_id);
    assert_eq!(node.pubkey, pubkey);
}

#[test]
fn test_update_node_logic_success() {
    let mut node = OracleNode { node_id: 1, pubkey: Pubkey::new_unique() };
    let new_id = 99;
    let new_pubkey = Pubkey::new_unique();
    update_node_logic(&mut node, new_id, new_pubkey);
    assert_eq!(node.node_id, new_id);
    assert_eq!(node.pubkey, new_pubkey);
}

#[test]
fn test_initialize_logic_success() {
    let mut config = OracleConfig {
        admin: Pubkey::default(),
        chain_whitelist: [0; 8],
        oracle_nodes: [Pubkey::default(); 8],
        sender_whitelist: [Pubkey::default(); 8],
        is_initialized: false,
    };
    let authority = Pubkey::new_unique();
    let chain_ids = [1,2,3,4,5,6,7,8];
    let node_pubkeys = [Pubkey::new_unique(); 8];
    let result = initialize_logic(&mut config, &authority, chain_ids, node_pubkeys);
    assert!(result.is_ok());
    assert_eq!(config.admin, authority);
    assert_eq!(config.chain_whitelist, chain_ids);
    assert_eq!(config.oracle_nodes, node_pubkeys);
    assert!(config.is_initialized);
}

#[test]
fn test_initialize_logic_fail_already_initialized() {
    let mut config = OracleConfig {
        admin: Pubkey::default(),
        chain_whitelist: [0; 8],
        oracle_nodes: [Pubkey::default(); 8],
        sender_whitelist: [Pubkey::default(); 8],
        is_initialized: true,
    };
    let authority = Pubkey::new_unique();
    let chain_ids = [1,2,3,4,5,6,7,8];
    let node_pubkeys = [Pubkey::new_unique(); 8];
    let result = initialize_logic(&mut config, &authority, chain_ids, node_pubkeys);
    assert!(result.is_err());
}

#[test]
fn test_set_admin_logic_success() {
    let mut config = OracleConfig {
        admin: Pubkey::new_unique(),
        chain_whitelist: [0; 8],
        oracle_nodes: [Pubkey::default(); 8],
        sender_whitelist: [Pubkey::default(); 8],
        is_initialized: true,
    };
    let authority = config.admin;
    let new_admin = Pubkey::new_unique();
    let result = set_admin_logic(&mut config, &authority, new_admin);
    assert!(result.is_ok());
    assert_eq!(config.admin, new_admin);
}

#[test]
fn test_set_admin_logic_fail_not_admin() {
    let mut config = OracleConfig {
        admin: Pubkey::new_unique(),
        chain_whitelist: [0; 8],
        oracle_nodes: [Pubkey::default(); 8],
        sender_whitelist: [Pubkey::default(); 8],
        is_initialized: true,
    };
    let fake = Pubkey::new_unique();
    let result = set_admin_logic(&mut config, &fake, Pubkey::new_unique());
    assert!(result.is_err());
}

#[test]
fn test_transmit_multisig_threshold() {
    // Move node public key definitions to the top
    let node1 = Pubkey::new_unique();
    let node2 = Pubkey::new_unique();
    let node3 = Pubkey::new_unique();
    // Define cross_chain_message and config first to ensure mock account initialization scope
    let cross_chain_message = create_cross_chain_message("msgid123", 0, 0);
    let mut oracle_nodes = [Pubkey::default(); 8];
    oracle_nodes[0] = node1;
    oracle_nodes[1] = node2;
    oracle_nodes[2] = node3;
    let mut config = create_oracle_config(Pubkey::new_unique(), true);
    config.oracle_nodes = oracle_nodes;
    // report_context token_transfer_metadata
    let report_context = ReportContext {
        message_id: "msgid123".to_string(),
        sender: "sender_contract".to_string(),
        receiver: Pubkey::new_unique(),
        src_chain: 1,
        dst_chain: 2,
        epoch: 1,
    };
    let report_context_bytes = report_context.try_to_vec().unwrap();
    let token_transfer_metadata = create_token_transfer_metadata(2, 100);
    let token_transfer_metadata_bytes = token_transfer_metadata.try_to_vec().unwrap();
    let message = b"hello world".to_vec();
    // Construct fake signatures and public keys (will not pass secp256k1 verification, but can test threshold logic)
    let signatures = vec![vec![1u8; 64], vec![2u8; 64]];
    let pubkeys = vec![node1.to_bytes().to_vec(), node2.to_bytes().to_vec()];
    let key3 = Pubkey::default();
    let mut lamports3 = 0u64;
    let mut data3 = vec![];
    let owner3 = Pubkey::default();
    let instructions_account = create_account_info(&key3, &mut lamports3, &mut data3, &owner3, &[], &[]);
    let data1 = {
        let mut data1 = vec![];
        let push_u32 = |v: u32, data: &mut Vec<u8>| { data.extend_from_slice(&v.to_le_bytes()); };
        push_u32(report_context_bytes.len() as u32, &mut data1);
        data1.extend_from_slice(&report_context_bytes);
        push_u32(message.len() as u32, &mut data1);
        data1.extend_from_slice(&message);
        push_u32(token_transfer_metadata_bytes.len() as u32, &mut data1);
        data1.extend_from_slice(&token_transfer_metadata_bytes);
        push_u32(signatures.len() as u32, &mut data1);
        for sig in &signatures {
            push_u32(sig.len() as u32, &mut data1);
            data1.extend_from_slice(sig);
        }
        data1
    };
    // --- Context construction ---
    let program_id = ID;
    let mut config_data = config.try_to_vec().unwrap();
    let mut config_lamports = 0u64;
    let config_key = Pubkey::new_unique();
    let config_owner = program_id;
    let oracle_config_account = create_account_info(
        &config_key,
        &mut config_lamports,
        &mut config_data,
        &config_owner,
        &OracleConfig::discriminator(),
        &config.try_to_vec().unwrap(),
    );
    let mut msg_data = CrossChainMessage::default().try_to_vec().unwrap();
    let mut msg_lamports = 0u64;
    let msg_key = Pubkey::new_unique();
    let msg_owner = program_id;
    let cross_chain_message_account = create_account_info(
        &msg_key,
        &mut msg_lamports,
        &mut msg_data,
        &msg_owner,
        &CrossChainMessage::discriminator(),
        &CrossChainMessage::default().try_to_vec().unwrap(),
    );
    let oracle_config_account = Account::<OracleConfig>::try_from(&oracle_config_account).unwrap();
    let cross_chain_message_account = Account::<CrossChainMessage>::try_from(&cross_chain_message_account).unwrap();
    let mut transmit = Transmit {
        oracle_config: oracle_config_account,
        cross_chain_message: cross_chain_message_account,
    };
    let accounts = vec![];
    let ctx = Context::new(&program_id, &mut transmit, &accounts[..], Default::default());
    let result = Oracle::transmit(ctx, data1.clone(), instructions_account.clone());
    assert!(result.is_err());
    // Test insufficient number of signatures
    let signatures = vec![vec![1u8; 64]];
    let data2 = {
        let mut data2 = vec![];
        let push_u32 = |v: u32, data: &mut Vec<u8>| { data.extend_from_slice(&v.to_le_bytes()); };
        push_u32(report_context_bytes.len() as u32, &mut data2);
        data2.extend_from_slice(&report_context_bytes);
        push_u32(message.len() as u32, &mut data2);
        data2.extend_from_slice(&message);
        push_u32(token_transfer_metadata_bytes.len() as u32, &mut data2);
        data2.extend_from_slice(&token_transfer_metadata_bytes);
        push_u32(signatures.len() as u32, &mut data2);
        for sig in &signatures {
            push_u32(sig.len() as u32, &mut data2);
            data2.extend_from_slice(sig);
        }
        data2
    };
    let ctx2 = Context::new(&program_id, &mut transmit, &accounts[..], Default::default());
    let result2 = Oracle::transmit(ctx2, data2.clone(), instructions_account.clone());
    assert!(result2.is_err());
}

#[test]
fn test_transmit_signature_below_threshold() {
    let node1 = Pubkey::new_unique();
    let node2 = Pubkey::new_unique();
    let mut oracle_nodes = [Pubkey::default(); 8];
    oracle_nodes[0] = node1;
    oracle_nodes[1] = node2;
    let mut config = create_oracle_config(Pubkey::new_unique(), true);
    config.oracle_nodes = oracle_nodes;
    let report_context = ReportContext::default();
    let report_context_bytes = report_context.try_to_vec().unwrap();
    let token_transfer_metadata = create_token_transfer_metadata(0, 0);
    let token_transfer_metadata_bytes = token_transfer_metadata.try_to_vec().unwrap();
    let message = b"test".to_vec();
    // Provide only one signature, threshold is 2
    let signatures = vec![vec![1u8; 64]];
    let pubkeys = vec![node1.to_bytes().to_vec()];
    let key3 = Pubkey::default();
    let mut lamports3 = 0u64;
    let mut data3 = vec![];
    let owner3 = Pubkey::default();
    let instructions_account = create_account_info(&key3, &mut lamports3, &mut data3, &owner3, &[], &[]);
    let data1 = {
        let mut data1 = vec![];
        let push_u32 = |v: u32, data: &mut Vec<u8>| { data.extend_from_slice(&v.to_le_bytes()); };
        push_u32(report_context_bytes.len() as u32, &mut data1);
        data1.extend_from_slice(&report_context_bytes);
        push_u32(message.len() as u32, &mut data1);
        data1.extend_from_slice(&message);
        push_u32(token_transfer_metadata_bytes.len() as u32, &mut data1);
        data1.extend_from_slice(&token_transfer_metadata_bytes);
        push_u32(signatures.len() as u32, &mut data1);
        for sig in &signatures {
            push_u32(sig.len() as u32, &mut data1);
            data1.extend_from_slice(sig);
        }
        data1
    };
    // --- Context construction ---
    let program_id = ID;
    let mut config_data = config.try_to_vec().unwrap();
    let mut config_lamports = 0u64;
    let config_key = Pubkey::new_unique();
    let config_owner = program_id;
    let oracle_config_account = create_account_info(
        &config_key,
        &mut config_lamports,
        &mut config_data,
        &config_owner,
        &OracleConfig::discriminator(),
        &config.try_to_vec().unwrap(),
    );
    let mut msg_data = CrossChainMessage::default().try_to_vec().unwrap();
    let mut msg_lamports = 0u64;
    let msg_key = Pubkey::new_unique();
    let msg_owner = program_id;
    let cross_chain_message_account = create_account_info(
        &msg_key,
        &mut msg_lamports,
        &mut msg_data,
        &msg_owner,
        &CrossChainMessage::discriminator(),
        &CrossChainMessage::default().try_to_vec().unwrap(),
    );
    let oracle_config_account = Account::<OracleConfig>::try_from(&oracle_config_account).unwrap();
    let cross_chain_message_account = Account::<CrossChainMessage>::try_from(&cross_chain_message_account).unwrap();
    let mut transmit = Transmit {
        oracle_config: oracle_config_account,
        cross_chain_message: cross_chain_message_account,
    };
    let accounts = vec![];
    let ctx = Context::new(&program_id, &mut transmit, &accounts[..], Default::default());
    let result = Oracle::transmit(ctx, data1.clone(), instructions_account.clone());
    assert!(result.is_err());
}

#[test]
fn test_transmit_with_duplicate_signatures() {
    let node1 = Pubkey::new_unique();
    let node2 = Pubkey::new_unique();
    let mut oracle_nodes = [Pubkey::default(); 8];
    oracle_nodes[0] = node1;
    oracle_nodes[1] = node2;
    let config = OracleConfig {
        admin: Pubkey::new_unique(),
        chain_whitelist: [0; 8],
        oracle_nodes,
        sender_whitelist: [Pubkey::default(); 8],
        is_initialized: true,
    };
    let report_context = ReportContext::default();
    let report_context_bytes = report_context.try_to_vec().unwrap();
    let token_transfer_metadata = TokenTransferMetadata::default();
    let token_transfer_metadata_bytes = token_transfer_metadata.try_to_vec().unwrap();
    let message = b"test".to_vec();
    // Both signatures are from node1, duplicated
    let signatures = vec![vec![1u8; 64], vec![2u8; 64]];
    let pubkeys = vec![node1.to_bytes().to_vec(), node1.to_bytes().to_vec()];
    let key3 = Pubkey::default();
    let mut lamports3 = 0u64;
    let mut data3 = vec![];
    let owner3 = Pubkey::default();
    let instructions_account = create_account_info(&key3, &mut lamports3, &mut data3, &owner3, &[], &[]);
    let data1 = {
        let mut data1 = vec![];
        let push_u32 = |v: u32, data: &mut Vec<u8>| { data.extend_from_slice(&v.to_le_bytes()); };
        push_u32(report_context_bytes.len() as u32, &mut data1);
        data1.extend_from_slice(&report_context_bytes);
        push_u32(message.len() as u32, &mut data1);
        data1.extend_from_slice(&message);
        push_u32(token_transfer_metadata_bytes.len() as u32, &mut data1);
        data1.extend_from_slice(&token_transfer_metadata_bytes);
        push_u32(signatures.len() as u32, &mut data1);
        for sig in &signatures {
            push_u32(sig.len() as u32, &mut data1);
            data1.extend_from_slice(sig);
        }
        data1
    };
    // --- Context construction ---
    let program_id = ID;
    let mut config_data = config.try_to_vec().unwrap();
    let mut config_lamports = 0u64;
    let config_key = Pubkey::new_unique();
    let config_owner = program_id;
    let oracle_config_account = create_account_info(
        &config_key,
        &mut config_lamports,
        &mut config_data,
        &config_owner,
        &OracleConfig::discriminator(),
        &config.try_to_vec().unwrap(),
    );
    let mut msg_data = CrossChainMessage::default().try_to_vec().unwrap();
    let mut msg_lamports = 0u64;
    let msg_key = Pubkey::new_unique();
    let msg_owner = program_id;
    let cross_chain_message_account = create_account_info(
        &msg_key,
        &mut msg_lamports,
        &mut msg_data,
        &msg_owner,
        &CrossChainMessage::discriminator(),
        &CrossChainMessage::default().try_to_vec().unwrap(),
    );
    let oracle_config_account = Account::<OracleConfig>::try_from(&oracle_config_account).unwrap();
    let cross_chain_message_account = Account::<CrossChainMessage>::try_from(&cross_chain_message_account).unwrap();
    let mut transmit = Transmit {
        oracle_config: oracle_config_account,
        cross_chain_message: cross_chain_message_account,
    };
    let accounts = vec![];
    let ctx = Context::new(&program_id, &mut transmit, &accounts[..], Default::default());
    let result = Oracle::transmit(ctx, data1.clone(), instructions_account.clone());
    // Threshold is 2, only 1 valid signature after deduplication
    assert!(result.is_err());
}

#[test]
fn test_transmit_with_too_many_signatures() {
    let node1 = Pubkey::new_unique();
    let node2 = Pubkey::new_unique();
    let mut oracle_nodes = [Pubkey::default(); 8];
    oracle_nodes[0] = node1;
    oracle_nodes[1] = node2;
    let config = OracleConfig {
        admin: Pubkey::new_unique(),
        chain_whitelist: [0; 8],
        oracle_nodes,
        sender_whitelist: [Pubkey::default(); 8],
        is_initialized: true,
    };
    let report_context = ReportContext::default();
    let report_context_bytes = report_context.try_to_vec().unwrap();
    let token_transfer_metadata = TokenTransferMetadata::default();
    let token_transfer_metadata_bytes = token_transfer_metadata.try_to_vec().unwrap();
    let message = b"test".to_vec();
    // Number of signatures exceeds number of nodes
    let signatures = vec![vec![1u8; 64], vec![2u8; 64], vec![3u8; 64], vec![4u8; 64]];
    let pubkeys = vec![node1.to_bytes().to_vec(), node2.to_bytes().to_vec(), node1.to_bytes().to_vec(), node2.to_bytes().to_vec()];
    let key3 = Pubkey::default();
    let mut lamports3 = 0u64;
    let mut data3 = vec![];
    let owner3 = Pubkey::default();
    let instructions_account = create_account_info(&key3, &mut lamports3, &mut data3, &owner3, &[], &[]);
    let data1 = {
        let mut data1 = vec![];
        let push_u32 = |v: u32, data: &mut Vec<u8>| { data.extend_from_slice(&v.to_le_bytes()); };
        push_u32(report_context_bytes.len() as u32, &mut data1);
        data1.extend_from_slice(&report_context_bytes);
        push_u32(message.len() as u32, &mut data1);
        data1.extend_from_slice(&message);
        push_u32(token_transfer_metadata_bytes.len() as u32, &mut data1);
        data1.extend_from_slice(&token_transfer_metadata_bytes);
        push_u32(signatures.len() as u32, &mut data1);
        for sig in &signatures {
            push_u32(sig.len() as u32, &mut data1);
            data1.extend_from_slice(sig);
        }
        data1
    };
    // --- Context construction ---
    let program_id = ID;
    let mut config_data = config.try_to_vec().unwrap();
    let mut config_lamports = 0u64;
    let config_key = Pubkey::new_unique();
    let config_owner = program_id;
    let oracle_config_account = create_account_info(
        &config_key,
        &mut config_lamports,
        &mut config_data,
        &config_owner,
        &OracleConfig::discriminator(),
        &config.try_to_vec().unwrap(),
    );
    let mut msg_data = CrossChainMessage::default().try_to_vec().unwrap();
    let mut msg_lamports = 0u64;
    let msg_key = Pubkey::new_unique();
    let msg_owner = program_id;
    let cross_chain_message_account = create_account_info(
        &msg_key,
        &mut msg_lamports,
        &mut msg_data,
        &msg_owner,
        &CrossChainMessage::discriminator(),
        &CrossChainMessage::default().try_to_vec().unwrap(),
    );
    let oracle_config_account = Account::<OracleConfig>::try_from(&oracle_config_account).unwrap();
    let cross_chain_message_account = Account::<CrossChainMessage>::try_from(&cross_chain_message_account).unwrap();
    let mut transmit = Transmit {
        oracle_config: oracle_config_account,
        cross_chain_message: cross_chain_message_account,
    };
    let accounts = vec![];
    let ctx = Context::new(&program_id, &mut transmit, &accounts[..], Default::default());
    let result = Oracle::transmit(ctx, data1.clone(), instructions_account.clone());
    // As long as the number of valid signatures does not exceed the threshold, it will still error
    assert!(result.is_err());
} 