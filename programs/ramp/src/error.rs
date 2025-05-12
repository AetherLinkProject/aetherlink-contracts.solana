use anchor_lang::prelude::*;

#[error_code]
pub enum RampError {
    #[msg("Already initialized")]
    AlreadyInitialized,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Sender whitelist full")]
    SenderWhitelistFull,
    #[msg("Sender not found")]
    SenderNotFound,
} 