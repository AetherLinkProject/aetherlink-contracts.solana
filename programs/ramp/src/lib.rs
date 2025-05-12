use anchor_lang::prelude::*;
use anchor_lang::{AnchorSerialize, AnchorDeserialize};
mod context;
pub mod error;
use context::*;
use error::RampError;
mod events;
use events::*;

declare_id!("37dWZXiyYBZJvPN4UbAJ1BGEUuUyMFw5dnmSMVTMswfr");

#[account]
pub struct RampConfig {
    pub is_initialized: bool,
    pub authority: Pubkey,
    pub oracle_nodes: [Pubkey; 8], // 新增节点集字段
    pub sender_whitelist: [Pubkey; 8], // 新增 sender 白名单
    pub source_chain_whitelist: [u64; 8], // 来源链白名单
    pub destination_chain_whitelist: [u64; 8], // 目标链白名单
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Debug)]
pub struct TokenTransferMetadata {
    pub target_chain_id: u64,
    pub token_address: Pubkey,
    pub symbol: [u8; 16],
    pub amount: u64,
    pub extra_data: String,
}

#[program]
pub mod ramp {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>, authority: Pubkey) -> Result<()> {
        let ramp_config = &mut ctx.accounts.ramp_config;
        require!(!ramp_config.is_initialized, RampError::AlreadyInitialized);
        ramp_config.is_initialized = true;
        ramp_config.authority = authority;
        let slot = Clock::get()?.slot;
        emit!(Initialized {
            authority,
            slot,
        });
        Ok(())
    }

    pub fn set_admin(ctx: Context<SetAdmin>, new_authority: Pubkey) -> Result<()> {
        let ramp_config = &mut ctx.accounts.ramp_config;
        require!(ctx.accounts.current_authority.key() == ramp_config.authority, RampError::Unauthorized);
        let old_authority = ramp_config.authority;
        ramp_config.authority = new_authority;
        let slot = Clock::get()?.slot;
        emit!(AdminUpdated {
            old_authority,
            new_authority,
            slot,
        });
        Ok(())
    }

    pub fn set_oracle_nodes(ctx: Context<SetOracleNodes>, node_pubkeys: [Pubkey; 8]) -> Result<()> {
        let ramp_config = &mut ctx.accounts.ramp_config;
        require!(ctx.accounts.authority.key() == ramp_config.authority, RampError::Unauthorized);
        ramp_config.oracle_nodes = node_pubkeys;
        let slot = Clock::get()?.slot;
        emit!(OracleNodesUpdated {
            authority: ctx.accounts.authority.key(),
            node_pubkeys,
            slot,
        });
        Ok(())
    }

    pub fn add_sender(ctx: Context<AddSender>, sender: Pubkey) -> Result<()> {
        let ramp_config = &mut ctx.accounts.ramp_config;
        require!(ctx.accounts.authority.key() == ramp_config.authority, RampError::Unauthorized);
        for slot in ramp_config.sender_whitelist.iter_mut() {
            if *slot == Pubkey::default() {
                *slot = sender;
                let slot_num = Clock::get()?.slot;
                emit!(SenderAdded {
                    authority: ctx.accounts.authority.key(),
                    sender,
                    slot: slot_num,
                });
                return Ok(());
            }
            if *slot == sender {
                let slot_num = Clock::get()?.slot;
                emit!(SenderAdded {
                    authority: ctx.accounts.authority.key(),
                    sender,
                    slot: slot_num,
                });
                return Ok(()); // already whitelisted
            }
        }
        Err(RampError::SenderWhitelistFull.into())
    }

    pub fn remove_sender(ctx: Context<RemoveSender>, sender: Pubkey) -> Result<()> {
        let ramp_config = &mut ctx.accounts.ramp_config;
        require!(ctx.accounts.authority.key() == ramp_config.authority, RampError::Unauthorized);
        for slot in ramp_config.sender_whitelist.iter_mut() {
            if *slot == sender {
                *slot = Pubkey::default();
                let slot_num = Clock::get()?.slot;
                emit!(SenderRemoved {
                    authority: ctx.accounts.authority.key(),
                    sender,
                    slot: slot_num,
                });
                return Ok(());
            }
        }
        Err(RampError::SenderNotFound.into())
    }

    pub fn set_chain_whitelist(ctx: Context<SetChainWhitelist>, source_chain_ids: [u64; 8], destination_chain_ids: [u64; 8]) -> Result<()> {
        let ramp_config = &mut ctx.accounts.ramp_config;
        require!(ctx.accounts.authority.key() == ramp_config.authority, RampError::Unauthorized);
        ramp_config.source_chain_whitelist = source_chain_ids;
        ramp_config.destination_chain_whitelist = destination_chain_ids;
        let slot = Clock::get()?.slot;
        emit!(ChainWhitelistUpdated {
            authority: ctx.accounts.authority.key(),
            source_chain_ids,
            destination_chain_ids,
            slot,
        });
        Ok(())
    }

    pub fn transmit(ctx: Context<Transmit>, data: Vec<u8>) -> Result<()> {
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
        // --- chainlink风格验签 ---
        let mut hasher = anchor_lang::solana_program::keccak::Hasher::default();
        hasher.hash(&report_context_bytes);
        hasher.hash(&message);
        hasher.hash(&token_transfer_metadata_bytes);
        let hash = hasher.result(); // [u8; 32]
        let ramp_config = &ctx.accounts.ramp_config;
        let mut unique_validators = std::collections::HashSet::new();
        for i in 0..signatures.len() {
            let sig = &signatures[i];
            let rec_id = recovery_ids[i];
            if let Ok(pubkey) = anchor_lang::solana_program::secp256k1_recover::secp256k1_recover(hash.as_ref(), rec_id, sig) {
                for node_pk in ramp_config.oracle_nodes.iter() {
                    if *node_pk == Pubkey::default() { continue; }
                    if node_pk.to_bytes() == pubkey.0[0..32] {
                        unique_validators.insert(*node_pk);
                    }
                }
            }
        }
        let threshold = (ramp_config.oracle_nodes.iter().filter(|x| **x != Pubkey::default()).count() / 2) + 1;
        require!(unique_validators.len() >= threshold, RampError::Unauthorized);
        let slot = Clock::get()?.slot;
        emit!(Transmitted {
            authority: ramp_config.authority,
            sig_count: signatures.len() as u32,
            unique_validators: unique_validators.len() as u32,
            slot,
        });
        Ok(())
    }

    pub fn send_request(ctx: Context<SendRequest>, dst_chain_id: u64, receiver: String, message: Vec<u8>, token_transfer_metadata: TokenTransferMetadata) -> Result<()> {
        let ramp_config = &ctx.accounts.ramp_config;
        let sender_key = ctx.accounts.sender.key();
        require!(ramp_config.sender_whitelist.contains(&sender_key), RampError::Unauthorized);
        require!(ramp_config.destination_chain_whitelist.contains(&dst_chain_id), RampError::Unauthorized);
        let slot = ctx.accounts.clock.slot;
        let mut hasher = anchor_lang::solana_program::keccak::Hasher::default();
        hasher.hash(sender_key.as_ref());
        hasher.hash(&dst_chain_id.to_le_bytes());
        hasher.hash(receiver.as_bytes());
        hasher.hash(&message);
        hasher.hash(&token_transfer_metadata.try_to_vec().unwrap());
        hasher.hash(&slot.to_le_bytes());
        let message_id = hasher.result().to_bytes();
        msg!("SendRequest: sender={}, dst_chain_id={}, receiver={}, message={:?}, token_transfer_metadata={:?}, slot={}, message_id={:?}", sender_key, dst_chain_id, receiver, message, token_transfer_metadata, slot, message_id);
        emit!(RequestSend {
            sender: sender_key,
            dst_chain_id,
            receiver: receiver.clone(),
            message_id,
            slot,
            token_transfer_metadata: token_transfer_metadata.clone(),
        });
        Ok(())
    }
} 