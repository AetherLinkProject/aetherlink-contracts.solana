use anchor_lang::prelude::*;
use crate::account_types::oracle_node::OracleNode;

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
}

pub fn handler(ctx: Context<RegisterNode>, node_id: u64, metadata: [u8; 64]) -> Result<()> {
    let node = &mut ctx.accounts.oracle_node;
    node.authority = *ctx.accounts.authority.key;
    node.node_id = node_id;
    node.status = 1; // activated
    node.metadata = metadata;
    node.bump = ctx.bumps.oracle_node;
    Ok(())
} 