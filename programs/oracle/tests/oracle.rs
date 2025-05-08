use oracle::*;
use oracle::OracleConfig;
use oracle::TokenTransferMetadata;
use anchor_lang::prelude::*;
use secp256k1::{Secp256k1, SecretKey, PublicKey, Message, ecdsa::Signature};
use rand::rngs::OsRng;
use solana_sdk::keccak::hashv as keccak;

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
    let target_chain_id = 42u64;
    let receiver = Pubkey::new_unique();
    let message = b"test message";
    let token_transfer_metadata = TokenTransferMetadata {
        target_chain_id,
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
        target_chain_id,
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
        target_chain_id,
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