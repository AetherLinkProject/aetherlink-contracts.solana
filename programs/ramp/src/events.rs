use anchor_lang::prelude::*;
use crate::TokenTransferMetadata;

#[event]
pub struct RequestSend {
    pub sender: Pubkey,
    pub dst_chain_id: u64,
    pub receiver: String,
    pub message_id: [u8; 32],
    pub slot: u64,
    pub token_transfer_metadata: TokenTransferMetadata,
}

#[event]
pub struct Initialized {
    pub authority: Pubkey,
    pub slot: u64,
}

#[event]
pub struct AdminUpdated {
    pub old_authority: Pubkey,
    pub new_authority: Pubkey,
    pub slot: u64,
}

#[event]
pub struct OracleNodesUpdated {
    pub authority: Pubkey,
    pub node_pubkeys: [Pubkey; 8],
    pub slot: u64,
}

#[event]
pub struct SenderAdded {
    pub authority: Pubkey,
    pub sender: Pubkey,
    pub slot: u64,
}

#[event]
pub struct SenderRemoved {
    pub authority: Pubkey,
    pub sender: Pubkey,
    pub slot: u64,
}

#[event]
pub struct ChainWhitelistUpdated {
    pub authority: Pubkey,
    pub source_chain_ids: [u64; 8],
    pub destination_chain_ids: [u64; 8],
    pub slot: u64,
}

#[event]
pub struct Transmitted {
    pub authority: Pubkey,
    pub sig_count: u32,
    pub unique_validators: u32,
    pub slot: u64,
} 