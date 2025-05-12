use anchor_lang::prelude::*;

#[account]
pub struct RampConfig {
    pub is_initialized: bool,
    pub authority: Pubkey,
    pub oracle_nodes: [Pubkey; 8],
    pub sender_whitelist: [Pubkey; 8],
    pub source_chain_whitelist: [u64; 8],
    pub destination_chain_whitelist: [u64; 8],
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 1 + 32 + 32*8 + 32*8 + 8*8 + 8*8,
        seeds = [b"ramp_config"],
        bump
    )]
    pub ramp_config: Account<'info, RampConfig>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetAdmin<'info> {
    #[account(mut, seeds = [b"ramp_config"], bump)]
    pub ramp_config: Account<'info, RampConfig>,
    pub current_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetOracleNodes<'info> {
    #[account(mut, seeds = [b"ramp_config"], bump)]
    pub ramp_config: Account<'info, RampConfig>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct AddSender<'info> {
    #[account(mut, seeds = [b"ramp_config"], bump)]
    pub ramp_config: Account<'info, RampConfig>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveSender<'info> {
    #[account(mut, seeds = [b"ramp_config"], bump)]
    pub ramp_config: Account<'info, RampConfig>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetChainWhitelist<'info> {
    #[account(mut, seeds = [b"ramp_config"], bump)]
    pub ramp_config: Account<'info, RampConfig>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Transmit<'info> {
    #[account(mut, seeds = [b"ramp_config"], bump)]
    pub ramp_config: Account<'info, RampConfig>,
    // 可扩展：消息或状态账户
    // pub message: Account<'info, MessageAccount>,
}

#[derive(Accounts)]
pub struct SendRequest<'info> {
    #[account(mut, seeds = [b"ramp_config"], bump)]
    pub ramp_config: Account<'info, RampConfig>,
    pub sender: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
    // 参数: dst_chain_id, receiver: String, message: Vec<u8>, token_transfer_metadata: TokenTransferMetadata
} 