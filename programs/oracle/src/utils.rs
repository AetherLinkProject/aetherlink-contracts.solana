use anchor_lang::prelude::*;

pub fn verify_signer(signer: &Pubkey, expected: &Pubkey) -> bool {
    signer == expected
} 