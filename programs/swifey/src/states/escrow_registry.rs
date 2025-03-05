use anchor_lang::prelude::*;
use crate::states::*;
use crate::errors::SwifeyError;

/// Maximum number of escrows to track per user
pub const MAX_ESCROWS_PER_USER: usize = 10;

#[account]
pub struct EscrowRegistry {
    /// The user this registry belongs to
    pub user: Pubkey,
    
    /// List of escrows where this user is the initiator
    pub initiated_escrows: Vec<Pubkey>,
    
    /// List of escrows where this user is the beneficiary
    pub beneficiary_escrows: Vec<Pubkey>,
    
    /// Bump seed for PDA derivation
    pub bump: u8,
}

impl EscrowRegistry {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        4 + (32 * MAX_ESCROWS_PER_USER) + // initiated_escrows (vec with max capacity)
        4 + (32 * MAX_ESCROWS_PER_USER) + // beneficiary_escrows (vec with max capacity)
        1;  // bump
        
    pub fn add_initiated_escrow(&mut self, escrow: Pubkey) -> Result<()> {
        require!(
            self.initiated_escrows.len() < MAX_ESCROWS_PER_USER,
            SwifeyError::TooManyEscrows
        );
        
        if !self.initiated_escrows.contains(&escrow) {
            self.initiated_escrows.push(escrow);
        }
        
        Ok(())
    }
    
    pub fn add_beneficiary_escrow(&mut self, escrow: Pubkey) -> Result<()> {
        require!(
            self.beneficiary_escrows.len() < MAX_ESCROWS_PER_USER,
            SwifeyError::TooManyEscrows
        );
        
        if !self.beneficiary_escrows.contains(&escrow) {
            self.beneficiary_escrows.push(escrow);
        }
        
        Ok(())
    }
    
    pub fn remove_escrow(&mut self, escrow: Pubkey) {
        // Remove from initiated escrows if present
        if let Some(index) = self.initiated_escrows.iter().position(|&e| e == escrow) {
            self.initiated_escrows.remove(index);
        }
        
        // Remove from beneficiary escrows if present
        if let Some(index) = self.beneficiary_escrows.iter().position(|&e| e == escrow) {
            self.beneficiary_escrows.remove(index);
        }
    }
}

/// Seeds for deriving the escrow registry PDA
pub const ESCROW_REGISTRY_SEED: &[u8] = b"escrow-registry"; 