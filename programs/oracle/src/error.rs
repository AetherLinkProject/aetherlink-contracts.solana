use anchor_lang::prelude::*;

#[error_code]
pub enum OracleError {
    #[msg("Invalid node metadata")] 
    InvalidMetadata,
    #[msg("Node already exists")] 
    NodeAlreadyExists,
    #[msg("Unauthorized operation")] 
    Unauthorized,
} 