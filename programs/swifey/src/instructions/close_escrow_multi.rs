use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, CloseAccount};
use crate::errors::SwifeyError;
use crate::states::*;
use crate::instructions::close_escrow::Distribution;

#[derive(Accounts)]
pub struct CloseEscrowMulti<'info> {
    #[account(
        constraint = (initiator.key() == escrow.initiator || initiator.key() == escrow.beneficiary) @ SwifeyError::UnauthorizedEscrowClose,
    )]
    pub initiator: Signer<'info>,
    
    #[account(
        mut,
        constraint = escrow.is_closed == false @ SwifeyError::EscrowAlreadyClosed,
        seeds = [
            ESCROW_SEED,
            escrow.initiator.as_ref(),
            escrow.beneficiary.as_ref(),
            escrow.mint.as_ref(),
        ],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,
    
    #[account(
        mut,
        constraint = escrow_token_account.key() == escrow.escrow_token_account @ SwifeyError::UnauthorizedAddress,
        constraint = escrow_token_account.mint == escrow.mint @ SwifeyError::InvalidPoolTokens,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [
            ESCROW_REGISTRY_SEED,
            escrow.initiator.as_ref(),
        ],
        bump,
    )]
    pub initiator_registry: Account<'info, EscrowRegistry>,
    
    #[account(
        mut,
        seeds = [
            ESCROW_REGISTRY_SEED,
            escrow.beneficiary.as_ref(),
        ],
        bump,
    )]
    pub beneficiary_registry: Account<'info, EscrowRegistry>,
    
    /// Remaining accounts should be token accounts for recipients
    /// They must be provided in the same order as the distributions
    
    pub token_program: Program<'info, Token>,
}

impl<'info> CloseEscrowMulti<'info> {
    pub fn process(
        &mut self,
        distribution_percentages: Vec<Distribution>,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<()> {
        // Check if escrow has expired
        if let Some(expires_at) = self.escrow.expires_at {
            let current_time = Clock::get()?.unix_timestamp;
            require!(current_time <= expires_at, SwifeyError::EscrowExpired);
        }
        
        // Validate distribution percentages (should sum to 100%)
        let total_percentage: u64 = distribution_percentages.iter()
            .map(|d| d.amount)
            .sum();
            
        require!(
            total_percentage == 100,
            SwifeyError::InvalidDistributionAmounts
        );
        
        // Ensure we have the correct number of remaining accounts
        require!(
            remaining_accounts.len() == distribution_percentages.len(),
            SwifeyError::InvalidDistributionRecipient
        );
        
        // Mark escrow as closed
        self.escrow.is_closed = true;
        
        // Prepare escrow signer seeds
        let escrow_seeds = &[
            ESCROW_SEED,
            self.escrow.initiator.as_ref(),
            self.escrow.beneficiary.as_ref(),
            self.escrow.mint.as_ref(),
            &[self.escrow.bump],
        ];
        
        let signer_seeds = &[&escrow_seeds[..]];
        
        // Transfer tokens to each recipient based on percentage
        for (distribution, recipient_account) in distribution_percentages.iter()
            .zip(remaining_accounts.iter()) {
            
            // Calculate amount based on percentage
            let amount = (self.escrow.amount * distribution.amount) / 100;
            
            // Skip zero amount distributions
            if amount == 0 {
                continue;
            }
            
            // Verify the recipient token account
            let recipient_token_account = Account::<TokenAccount>::try_from(recipient_account)?;
            
            require!(
                recipient_token_account.owner == distribution.recipient,
                SwifeyError::InvalidDistributionRecipient
            );
            
            // Transfer tokens from escrow to recipient
            let cpi_accounts = Transfer {
                from: self.escrow_token_account.to_account_info(),
                to: recipient_account.clone(),
                authority: self.escrow.to_account_info(),
            };
            
            let cpi_program = self.token_program.to_account_info();
            let cpi_ctx = CpiContext::new_with_signer(
                cpi_program.clone(),
                cpi_accounts,
                signer_seeds,
            );
            
            token::transfer(cpi_ctx, amount)?;
        }
        
        // Close the escrow token account and return rent to initiator
        let cpi_accounts = CloseAccount {
            account: self.escrow_token_account.to_account_info(),
            destination: self.initiator.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
        
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(
            cpi_program,
            cpi_accounts,
            signer_seeds,
        );
        
        token::close_account(cpi_ctx)?;
        
        // Update registries
        self.initiator_registry.remove_escrow(self.escrow.key());
        self.beneficiary_registry.remove_escrow(self.escrow.key());
        
        Ok(())
    }
}

pub fn close_escrow_multi<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, CloseEscrowMulti<'info>>,
    distribution_percentages: Vec<Distribution>,
) -> Result<()> {
    ctx.accounts.process(distribution_percentages, ctx.remaining_accounts.as_ref())
} 