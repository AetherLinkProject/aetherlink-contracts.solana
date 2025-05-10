// SPDX-License-Identifier: MIT
// Anchor Oracle Program - Unified Structure

use anchor_lang::prelude::*;
use anchor_lang::prelude::{AnchorSerialize, AnchorDeserialize};
use anchor_lang::error_code;
use solana_program::sysvar::clock::Clock;
use anchor_lang::solana_program::sysvar;
use solana_program::sysvar::instructions::{load_current_index_checked, load_instruction_at_checked};
use sha2::Sha256;
use sha2::Digest;
use hex;
use anchor_lang::solana_program::{keccak, secp256k1_recover::secp256k1_recover};

// declare_id for the program

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkgMQHGt1hFJk");

// --- Account Structs ---

#[account]
#[derive(Debug)]
pub struct OracleNode {
    pub node_id: u64,    // Node index
    pub pubkey: Pubkey,  // Node public key
}

impl Default for OracleNode {
    fn default() -> Self {
        Self {
            node_id: 0,
            pubkey: Pubkey::default(),
        }
    }
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ReportContext {
    pub message_id: String,   // Cross-chain message unique id
    pub sender: String,      // Source chain contract
    pub receiver: Pubkey,    // Next contract address
    pub src_chain: u64,      // Source chain ID
    pub dst_chain: u64,      // Destination chain ID
    pub epoch: u64,          // Cross-chain epoch
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct TokenTransferMetadata {
    pub target_chain_id: u64,    // Destination chain ID
    pub token_address: Pubkey,   // Token address
    pub symbol: [u8; 16],        // Token symbol, fixed 16 bytes
    pub amount: u64,             // Transfer amount
    pub extra_data: String,      // Extra data
}

#[account]
#[derive(Debug)]
pub struct CrossChainMessage {
    pub message_id: String, // Cross-chain message unique id
    pub status: u8,         // message status (0=pending, 1=completed, 2=failed)
    pub bump: u8,
}

impl Default for CrossChainMessage {
    fn default() -> Self {
        Self {
            message_id: String::new(),
            status: 0,
            bump: 0,
        }
    }
}

#[account]
pub struct EpochState {
    pub epoch: u64,
    pub bump: u8,
}

#[account]
#[derive(Debug, PartialEq, Eq)]
pub struct OracleConfig {
    pub admin: Pubkey, // Global admin
    pub chain_whitelist: [u64; 8], // Fixed-length chain whitelist
    pub oracle_nodes: [Pubkey; 8], // Fixed-length oracle node set
    pub sender_whitelist: [Pubkey; 8], // Fixed-length sender whitelist
    pub is_initialized: bool, // Initialization flag
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct DecryptedReport {
    pub round_id: u64,
    pub answer: i128,
    pub timestamp: u64,
}

// --- Context/Accounts Structs ---

#[derive(Accounts)]
pub struct RegisterNode<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<OracleNode>(),
        seeds = [b"oracle_node", authority.key().as_ref()],
        bump
    )]
    pub oracle_node: Account<'info, OracleNode>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub oracle_config: Account<'info, OracleConfig>,
}

#[derive(Accounts)]
pub struct SendRequest<'info> {
    /// CHECK: sender whitelist PDA
    pub sender_whitelist: AccountInfo<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account(address = sysvar::clock::ID)]
    pub clock: AccountInfo<'info>,
    #[account(mut, seeds = [b"epoch_state"], bump = epoch_state.bump)]
    pub epoch_state: Account<'info, EpochState>,
    pub oracle_config: Account<'info, OracleConfig>,
}

#[derive(Accounts)]
pub struct ReceiveMessage<'info> {
    pub chain_message: Account<'info, CrossChainMessage>,
    pub receiver: Signer<'info>,
    pub oracle_config: Account<'info, OracleConfig>,
}

#[derive(Accounts)]
pub struct UpdateNode<'info> {
    pub oracle_node: Account<'info, OracleNode>,
    pub authority: Signer<'info>,
    pub oracle_config: Account<'info, OracleConfig>,
}

#[derive(Accounts)]
pub struct SetChainWhitelist<'info> {
    #[account(mut)]
    pub oracle_config: Account<'info, OracleConfig>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetOracleNodes<'info> {
    #[account(mut)]
    pub oracle_config: Account<'info, OracleConfig>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct InitEpochState<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 8 + 1, // discriminator + u64 + u8
        seeds = [b"epoch_state"],
        bump
    )]
    pub epoch_state: Account<'info, EpochState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitOracleConfig<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8*8 + 32*8 + 32*8 + 1, // discriminator + Pubkey + 8 u64 + 8 Pubkey + 8 sender Pubkey + bool
        seeds = [b"oracle_config"],
        bump
    )]
    pub oracle_config: Account<'info, OracleConfig>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetAdmin<'info> {
    #[account(mut)]
    pub oracle_config: Account<'info, OracleConfig>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct AddSender<'info> {
    #[account(mut)]
    pub oracle_config: Account<'info, OracleConfig>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveSender<'info> {
    #[account(mut)]
    pub oracle_config: Account<'info, OracleConfig>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Transmit<'info> {
    #[account(mut)]
    pub oracle_config: Account<'info, OracleConfig>,
    #[account(mut)]
    pub cross_chain_message: Account<'info, CrossChainMessage>,
    // Extendable for other contract state accounts
}

// --- Error Codes ---

#[error_code]
pub enum OracleError {
    #[msg("Invalid node metadata")] 
    InvalidMetadata,
    #[msg("Node already exists")] 
    NodeAlreadyExists,
    #[msg("Unauthorized operation")] 
    Unauthorized,
    #[msg("Invalid whitelist action")] 
    InvalidWhitelistAction,
    #[msg("Duplicate message_id submitted")]
    DuplicateMessageId,
    #[msg("Invalid sender address")]
    InvalidSenderAddress,
}

// --- Logic Functions ---

pub fn initialize_logic(config: &mut OracleConfig, authority: &Pubkey, chain_ids: [u64; 8], node_pubkeys: [Pubkey; 8]) -> Result<()> {
    require!(!config.is_initialized, OracleError::InvalidMetadata);
    config.admin = *authority;
    config.chain_whitelist = chain_ids;
    config.oracle_nodes = node_pubkeys;
    config.is_initialized = true;
    Ok(())
}

pub fn set_admin_logic(config: &mut OracleConfig, authority: &Pubkey, admin: Pubkey) -> Result<()> {
    require!(config.is_initialized, OracleError::InvalidMetadata);
    require!(*authority == config.admin, OracleError::Unauthorized);
    require!(admin != Pubkey::default(), OracleError::InvalidMetadata);
    msg!("Admin changed from {} to {}", config.admin, admin);
    config.admin = admin;
    Ok(())
}

pub fn set_chain_whitelist_logic(config: &mut OracleConfig, authority: &Pubkey, chain_ids: [u64; 8]) -> Result<()> {
    require!(*authority == config.admin, OracleError::Unauthorized);
    config.chain_whitelist = chain_ids;
    Ok(())
}

pub fn set_oracle_nodes_logic(config: &mut OracleConfig, authority: &Pubkey, node_pubkeys: [Pubkey; 8]) -> Result<()> {
    require!(*authority == config.admin, OracleError::Unauthorized);
    config.oracle_nodes = node_pubkeys;
    Ok(())
}

pub fn add_sender_logic(config: &mut OracleConfig, authority: &Pubkey, sender: Pubkey) -> Result<()> {
    require!(*authority == config.admin, OracleError::Unauthorized);
    for slot in config.sender_whitelist.iter_mut() {
        if *slot == Pubkey::default() {
            *slot = sender;
            return Ok(());
        }
        if *slot == sender {
            return Ok(()); // already whitelisted
        }
    }
    Err(OracleError::InvalidWhitelistAction.into())
}

pub fn remove_sender_logic(config: &mut OracleConfig, authority: &Pubkey, sender: Pubkey) -> Result<()> {
    require!(*authority == config.admin, OracleError::Unauthorized);
    for slot in config.sender_whitelist.iter_mut() {
        if *slot == sender {
            *slot = Pubkey::default();
            return Ok(());
        }
    }
    Err(OracleError::InvalidWhitelistAction.into())
}

pub fn send_request_logic(
    payer: &Pubkey,
    chain_whitelist: &[u64; 8],
    sender_whitelist: &[Pubkey; 8],
    sender: &str,
    dst_chain_id: u64,
    receiver: &str,
    message: &[u8],
    token_transfer_metadata: &TokenTransferMetadata,
    epoch: u64,
    blocktime: i64,
) -> Result<String> {
    let mut valid_sender = false;
    for s in sender_whitelist.iter() {
        if *s == *payer {
            valid_sender = true;
            break;
        }
    }
    require!(valid_sender, OracleError::InvalidSenderAddress);
    let mut whitelisted = false;
    for chain_id in chain_whitelist.iter() {
        if *chain_id == dst_chain_id { whitelisted = true; break; }
    }
    require!(whitelisted, OracleError::Unauthorized);
    // messageId: keccak256(abi.encodePacked(sender, dstChainId, receiver, message, tokenTransferMetadata, epoch, blocktime))
    let mut hasher = sha2::Sha256::new();
    hasher.update(sender.as_bytes());
    hasher.update(&dst_chain_id.to_le_bytes());
    hasher.update(receiver.as_bytes());
    hasher.update(message);
    hasher.update(&token_transfer_metadata.try_to_vec().unwrap());
    hasher.update(&epoch.to_le_bytes());
    hasher.update(&blocktime.to_le_bytes());
    let message_id = hex::encode(hasher.finalize());
    msg!("Event: RequestSent {{ message_id: {}, sender: {}, dst_chain_id: {}, receiver: {}, message: {:?}, token_transfer_metadata: {:?}, epoch: {}, blocktime: {} }}",
        message_id, sender, dst_chain_id, receiver, message, token_transfer_metadata, epoch, blocktime);
    Ok(message_id)
}

/// Mock 解密方法，结构对齐 Chainlink OCR2 合约
pub fn decrypt_report(data: &[u8]) -> Result<DecryptedReport, OracleError> {
    // 伪解密逻辑：直接从 data 取部分字节填充结构
    if data.len() < 24 {
        return Err(OracleError::InvalidMetadata.into());
    }
    let round_id = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let answer = i128::from_le_bytes(data[8..24].try_into().unwrap());
    let timestamp = if data.len() >= 32 {
        u64::from_le_bytes(data[24..32].try_into().unwrap())
    } else {
        0
    };
    Ok(DecryptedReport {
        round_id,
        answer,
        timestamp,
    })
}

// Program module forwarding
pub mod Oracle {
    use super::*;
    pub fn initialize(ctx: Context<InitOracleConfig>, chain_ids: [u64; 8], node_pubkeys: [Pubkey; 8]) -> Result<()> {
        initialize_logic(&mut ctx.accounts.oracle_config, &ctx.accounts.authority.key(), chain_ids, node_pubkeys)
    }
    pub fn set_admin(ctx: Context<SetAdmin>, admin: Pubkey) -> Result<()> {
        set_admin_logic(&mut ctx.accounts.oracle_config, &ctx.accounts.authority.key(), admin)
    }
    pub fn set_chain_whitelist(ctx: Context<SetChainWhitelist>, chain_ids: [u64; 8]) -> Result<()> {
        set_chain_whitelist_logic(&mut ctx.accounts.oracle_config, &ctx.accounts.authority.key(), chain_ids)
    }
    pub fn set_oracle_nodes(ctx: Context<SetOracleNodes>, node_pubkeys: [Pubkey; 8]) -> Result<()> {
        set_oracle_nodes_logic(&mut ctx.accounts.oracle_config, &ctx.accounts.authority.key(), node_pubkeys)
    }
    pub fn add_sender(ctx: Context<AddSender>, sender: Pubkey) -> Result<()> {
        add_sender_logic(&mut ctx.accounts.oracle_config, &ctx.accounts.authority.key(), sender)
    }
    pub fn remove_sender(ctx: Context<RemoveSender>, sender: Pubkey) -> Result<()> {
        remove_sender_logic(&mut ctx.accounts.oracle_config, &ctx.accounts.authority.key(), sender)
    }
    pub fn send_request(
        ctx: Context<SendRequest>,
        dst_chain_id: u64,
        receiver: String,
        message: Vec<u8>,
        token_transfer_metadata: TokenTransferMetadata,
    ) -> Result<()> {
        let sender = crate::ID.to_string();
        let _ = send_request_logic(
            &ctx.accounts.payer.key(),
            &ctx.accounts.oracle_config.chain_whitelist,
            &ctx.accounts.oracle_config.sender_whitelist,
            &sender,
            dst_chain_id,
            &receiver,
            &message,
            &token_transfer_metadata,
            ctx.accounts.epoch_state.epoch,
            Clock::from_account_info(&ctx.accounts.clock)?.unix_timestamp,
        )?;
        Ok(())
    }
    pub fn transmit(ctx: Context<Transmit>, data: Vec<u8>, _instructions_account: AccountInfo) -> Result<()> {
        // data: [rc_len|rc_bytes|msg_len|msg_bytes|meta_len|meta_bytes|sig_count|sig1(64)|rec_id1(1)|sig2|rec_id2|...]
        let mut offset = 0;
        let read_u32 = |data: &[u8], offset: &mut usize| -> u32 {
            let v = u32::from_le_bytes(data[*offset..*offset+4].try_into().unwrap());
            *offset += 4;
            v
        };
        let read_vec = |data: &[u8], offset: &mut usize, len: usize| -> Vec<u8> {
            let v = data[*offset..*offset+len].to_vec();
            *offset += len;
            v
        };
        let rc_len = read_u32(&data, &mut offset) as usize;
        let report_context_bytes = read_vec(&data, &mut offset, rc_len);
        let msg_len = read_u32(&data, &mut offset) as usize;
        let message = read_vec(&data, &mut offset, msg_len);
        let meta_len = read_u32(&data, &mut offset) as usize;
        let token_transfer_metadata_bytes = read_vec(&data, &mut offset, meta_len);
        let sig_count = read_u32(&data, &mut offset) as usize;
        let mut signatures = Vec::with_capacity(sig_count);
        let mut recovery_ids = Vec::with_capacity(sig_count);
        for _ in 0..sig_count {
            let sig = read_vec(&data, &mut offset, 64); // 64字节签名
            let rec_id = data[offset];
            offset += 1;
            signatures.push(sig);
            recovery_ids.push(rec_id);
        }
        let report_context = ReportContext::try_from_slice(&report_context_bytes)
            .map_err(|_| OracleError::InvalidMetadata)?;
        let _token_transfer_metadata = TokenTransferMetadata::try_from_slice(&token_transfer_metadata_bytes)
            .map_err(|_| OracleError::InvalidMetadata)?;
        require!(ctx.accounts.cross_chain_message.message_id == report_context.message_id, OracleError::InvalidMetadata);
        // --- chainlink风格验签 ---
        let mut hasher = keccak::Hasher::default();
        hasher.hash(&report_context_bytes);
        hasher.hash(&message);
        hasher.hash(&token_transfer_metadata_bytes);
        let hash = hasher.result(); // [u8; 32]
        let oracle_config = &ctx.accounts.oracle_config;
        let mut unique_validators = std::collections::HashSet::new();
        for i in 0..signatures.len() {
            let sig = &signatures[i];
            let rec_id = recovery_ids[i];
            if let Ok(pubkey) = secp256k1_recover(&hash, rec_id, sig) {
                // pubkey.0 为 64字节公钥
                for node_pk in oracle_config.oracle_nodes.iter() {
                    if *node_pk == Pubkey::default() { continue; }
                    if node_pk.to_bytes() == pubkey.0 {
                        unique_validators.insert(*node_pk);
                    }
                }
            }
        }
        let threshold = (oracle_config.oracle_nodes.iter().filter(|x| **x != Pubkey::default()).count() / 2) + 1;
        require!(unique_validators.len() >= threshold, OracleError::Unauthorized);
        let cross_chain_message = &mut ctx.accounts.cross_chain_message;
        if cross_chain_message.status != 0 {
            return Err(OracleError::DuplicateMessageId.into());
        }
        cross_chain_message.status = 1;
        msg!("Transmit success: message_id={}", report_context.message_id);
        Ok(())
    }
}

// --- Simple signature verification ---
pub fn verify_signature_secp256k1(
    message: &[u8],
    signature: &[u8],
    pubkey: &[u8],
    instructions_account: &AccountInfo,
) -> bool {
    let ix_index = match load_current_index_checked(instructions_account) {
        Ok(idx) => idx,
        Err(_) => {
            msg!("Failed to load current instruction index");
            return false;
        }
    };
    for i in 0..ix_index {
        let ix = match load_instruction_at_checked(i as usize, instructions_account) {
            Ok(ix) => ix,
            Err(_) => continue,
        };
        if ix.program_id == solana_program::secp256k1_program::ID {
            if ix.data.windows(pubkey.len()).any(|w| w == pubkey)
                && ix.data.windows(signature.len()).any(|w| w == signature)
                && ix.data.windows(message.len()).any(|w| w == message)
            {
                msg!("Found matching secp256k1 signature");
                return true;
            }
        }
    }
    msg!("No matching secp256k1 signature found");
    false
}

pub fn verify_signatures_secp256k1(
    message: &[u8],
    signatures: &[Vec<u8>],
    pubkeys: &[Vec<u8>],
    instructions_account: &AccountInfo,
) -> bool {
    if signatures.len() != pubkeys.len() {
        msg!("Signatures and pubkeys length mismatch");
        return false;
    }
    for (sig, pubkey) in signatures.iter().zip(pubkeys.iter()) {
        if !verify_signature_secp256k1(message, sig, pubkey, instructions_account) {
            msg!("Signature verification failed for one of the pairs");
            return false;
        }
    }
    true
} 