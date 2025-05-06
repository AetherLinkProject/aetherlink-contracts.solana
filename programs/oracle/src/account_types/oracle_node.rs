use anchor_lang::prelude::*;

#[account]
pub struct OracleNode {
    pub authority: Pubkey,      // Node owner
    pub node_id: u64,           // Unique node ID
    pub status: u8,             // Node status (0=inactive, 1=active, 2=disabled)
    pub metadata: [u8; 64],     // Node metadata (e.g., description, external ID, etc.)
    pub bump: u8,               // PDA bump
} 