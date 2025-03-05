use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Token};

#[account]
pub struct Escrow {
    /// The user who created the escrow
    pub initiator: Pubkey,
    
    /// The user for whom the escrow is created
    pub beneficiary: Pubkey,
    
    /// The token account that holds the escrowed funds
    pub escrow_token_account: Pubkey,
    
    /// The mint of the token being escrowed
    pub mint: Pubkey,
    
    /// The amount of tokens being escrowed
    pub amount: u64,
    
    /// Timestamp when the escrow was created
    pub created_at: i64,
    
    /// Optional expiry timestamp for the escrow
    pub expires_at: Option<i64>,
    
    /// Whether the escrow has been closed
    pub is_closed: bool,
    
    /// Bump seed for PDA derivation
    pub bump: u8,
}

impl Escrow {
    pub const LEN: usize = 8 + // discriminator
        32 + // initiator
        32 + // beneficiary
        32 + // escrow_token_account
        32 + // mint
        8 +  // amount
        8 +  // created_at
        9 +  // expires_at (Option<i64>)
        1 +  // is_closed
        1;   // bump
}

/// Seeds for deriving the escrow PDA
pub const ESCROW_SEED: &[u8] = b"escrow"; 