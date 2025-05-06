use anchor_lang::prelude::*;

#[account]
pub struct OracleConfig {
    pub admin: Pubkey,
    pub node_count: u64,
    pub min_signers: u8, // multisig threshold
    pub bump: u8,
} 