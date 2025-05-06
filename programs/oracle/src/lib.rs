use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkgMQHGt1hFJk");

#[account]
pub struct OracleNode {
    pub authority: Pubkey,      // node owner
    pub node_id: u64,           // unique node ID
    pub status: u8,             // node status (0=inactive, 1=active, 2=disabled)
    pub metadata: [u8; 64],     // node metadata (e.g., description, external ID, etc.)
    pub bump: u8,               // PDA bump
}

#[account]
pub struct SenderAuthority {
    pub authority: Pubkey, // initiator address
    pub enabled: bool,     // in whitelist
}

#[account]
pub struct ReceiverAuthority {
    pub authority: Pubkey, // receiver address
    pub enabled: bool,     // in whitelist
}

#[account]
pub struct ChainWhitelist {
    pub chain_id: u64,     // supported chain ID
    pub enabled: bool,     // in whitelist
}

#[account]
pub struct ChainMessage {
    pub sender: Pubkey,      // sender
    pub receiver: Pubkey,    // receiver
    pub src_chain: u64,      // source chain ID
    pub dst_chain: u64,      // destination chain ID
    pub payload: [u8; 128],  // cross-chain message content
    pub status: u8,          // message status (0=pending, 1=completed, 2=failed)
    pub bump: u8,
}

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

#[derive(Accounts)]
pub struct ManageWhitelist<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub whitelist_account: AccountInfo<'info>, // can be SenderAuthority/ReceiverAuthority/ChainWhitelist
}

#[derive(Accounts)]
pub struct SendRequest<'info> {
    #[account(
        init,
        payer = sender,
        space = 8 + std::mem::size_of::<ChainMessage>(),
        seeds = [b"chain_message", sender.key().as_ref(), receiver.key().as_ref()],
        bump
    )]
    pub chain_message: Account<'info, ChainMessage>,
    #[account(mut)]
    pub sender: Signer<'info>,
    /// CHECK: receiver can be any pubkey
    pub receiver: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ReceiveMessage<'info> {
    #[account(mut)]
    pub chain_message: Account<'info, ChainMessage>,
    #[account(mut)]
    pub receiver: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateNode<'info> {
    #[account(mut, has_one = authority)]
    pub oracle_node: Account<'info, OracleNode>,
    pub authority: Signer<'info>,
}

#[program]
pub mod oracle {
    use super::*;
    pub fn register_node(ctx: Context<RegisterNode>, node_id: u64, metadata: [u8; 64]) -> Result<()> {
        let node = &mut ctx.accounts.oracle_node;
        node.authority = *ctx.accounts.authority.key;
        node.node_id = node_id;
        node.status = 1; // activated
        node.metadata = metadata;
        node.bump = ctx.bumps.oracle_node;
        Ok(())
    }

    pub fn update_node(ctx: Context<UpdateNode>, new_status: u8, new_metadata: [u8; 64]) -> Result<()> {
        let node = &mut ctx.accounts.oracle_node;
        node.status = new_status;
        node.metadata = new_metadata;
        Ok(())
    }

    pub fn manage_whitelist(ctx: Context<ManageWhitelist>, enable: bool) -> Result<()> {
        let mut data = ctx.accounts.whitelist_account.try_borrow_mut_data()?;
        data[32] = enable as u8;
        Ok(())
    }

    pub fn send_request(
        ctx: Context<SendRequest>,
        src_chain: u64,
        dst_chain: u64,
        payload: [u8; 128],
    ) -> Result<()> {
        let msg = &mut ctx.accounts.chain_message;
        msg.sender = *ctx.accounts.sender.key;
        msg.receiver = *ctx.accounts.receiver.key;
        msg.src_chain = src_chain;
        msg.dst_chain = dst_chain;
        msg.payload = payload;
        msg.status = 0; // pending
        msg.bump = ctx.bumps.chain_message;
        Ok(())
    }

    pub fn receive_message(ctx: Context<ReceiveMessage>, status: u8) -> Result<()> {
        let msg = &mut ctx.accounts.chain_message;
        require!(msg.receiver == *ctx.accounts.receiver.key, CustomError::UnauthorizedReceiver);
        msg.status = status;
        Ok(())
    }
}

#[error_code]
pub enum CustomError {
    #[msg("Unauthorized message receiver")] 
    UnauthorizedReceiver,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn dummy_pubkey() -> Pubkey {
        Pubkey::new_unique()
    }

    #[test]
    fn test_register_node_normal() {
        let authority = Pubkey::new_unique();
        let mut node = OracleNode {
            authority,
            node_id: 1,
            status: 0,
            metadata: [0u8; 64],
            bump: 0,
        };
        let mut accounts = RegisterNode {
            oracle_node: Account::try_from(&mut node).unwrap(),
            authority: Signer::try_from(&authority).unwrap(),
            system_program: Program::try_from(&anchor_lang::solana_program::system_program::ID).unwrap(),
        };
        let ctx = Context::<RegisterNode> {
            accounts: &mut accounts,
            ..Default::default()
        };
        assert!(oracle::register_node(ctx, 1, [1u8; 64]).is_ok());
    }

    #[test]
    fn test_register_node_duplicate() {
        let authority = Pubkey::new_unique();
        let mut node = OracleNode {
            authority,
            node_id: 1,
            status: 1,
            metadata: [1u8; 64],
            bump: 0,
        };
        let mut accounts = RegisterNode {
            oracle_node: Account::try_from(&mut node).unwrap(),
            authority: Signer::try_from(&authority).unwrap(),
            system_program: Program::try_from(&anchor_lang::solana_program::system_program::ID).unwrap(),
        };
        let ctx = Context::<RegisterNode> {
            accounts: &mut accounts,
            ..Default::default()
        };
        assert!(oracle::register_node(ctx, 1, [1u8; 64]).is_err());
    }

    #[test]
    fn test_update_node_normal() {
        let authority = Pubkey::new_unique();
        let mut node = OracleNode {
            authority,
            node_id: 1,
            status: 1,
            metadata: [1u8; 64],
            bump: 0,
        };
        let mut accounts = UpdateNode {
            oracle_node: Account::try_from(&mut node).unwrap(),
            authority: Signer::try_from(&authority).unwrap(),
        };
        let ctx = Context::<UpdateNode> {
            accounts: &mut accounts,
            ..Default::default()
        };
        assert!(oracle::update_node(ctx, 2, [2u8; 64]).is_ok());
    }

    #[test]
    fn test_update_node_unauthorized() {
        let authority = Pubkey::new_unique();
        let other = Pubkey::new_unique();
        let mut node = OracleNode {
            authority,
            node_id: 1,
            status: 1,
            metadata: [1u8; 64],
            bump: 0,
        };
        let mut accounts = UpdateNode {
            oracle_node: Account::try_from(&mut node).unwrap(),
            authority: Signer::try_from(&other).unwrap(),
        };
        let ctx = Context::<UpdateNode> {
            accounts: &mut accounts,
            ..Default::default()
        };
        assert!(oracle::update_node(ctx, 2, [2u8; 64]).is_err());
    }

    #[test]
    fn test_send_request_normal() {
        // TODO: Construct SendRequest context, assert message creation success
    }

    #[test]
    fn test_receive_message_normal() {
        // TODO: Construct ReceiveMessage context, assert message status change
    }

    #[test]
    fn test_receive_message_unauthorized() {
        // TODO: Call receive_message as non-receiver, assert error
    }

    #[test]
    fn test_manage_whitelist_normal() {
        // TODO: Construct ManageWhitelist context, assert whitelist change
    }

    #[test]
    fn test_manage_whitelist_invalid() {
        // TODO: Call manage_whitelist as unauthorized, assert error
    }
} 