use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, CloseAccount};
use anchor_spl::associated_token::AssociatedToken;
use crate::errors::SwifeyError;
use crate::states::*;

/// Distribution represents a recipient and percentage for distributing escrowed funds
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Distribution {
    pub recipient: Pubkey,
    pub amount: u64,  // This is now a percentage (0-100)
}

#[derive(Accounts)]
pub struct CloseEscrow<'info> {
    #[account(
        mut,
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
        associated_token::mint = mint,
        associated_token::authority = escrow,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    /// The mint of the token being escrowed
    pub mint: Account<'info, anchor_spl::token::Mint>,
    
    #[account(
        mut,
        constraint = recipient_token_account.mint == escrow.mint @ SwifeyError::InvalidPoolTokens,
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,
    
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
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> CloseEscrow<'info> {
    pub fn process(
        &mut self,
        distribution_percentages: Vec<Distribution>,
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
        
        // Get first distribution and validate recipient token account
        let first_distribution = distribution_percentages.first().unwrap();
        
        // Validate that the recipient token account belongs to the recipient
        require!(
            self.recipient_token_account.owner == first_distribution.recipient,
            SwifeyError::InvalidDistributionRecipient
        );
        
        // Validate that the token account is an ATA
        let recipient_ata = anchor_spl::associated_token::get_associated_token_address(
            &first_distribution.recipient,
            &self.mint.key()
        );
        require!(
            self.recipient_token_account.key() == recipient_ata,
            SwifeyError::InvalidDistributionRecipient
        );
        
        // Mark escrow as closed
        self.escrow.is_closed = true;
        
        // Calculate amount to transfer
        let transfer_amount = self.escrow_token_account.amount;
        
        // Transfer tokens from escrow to recipient
        let escrow_seeds = &[
            ESCROW_SEED,
            self.escrow.initiator.as_ref(),
            self.escrow.beneficiary.as_ref(),
            self.escrow.mint.as_ref(),
            &[self.escrow.bump],
        ];
        
        let signer_seeds = &[&escrow_seeds[..]];
        
        // Transfer tokens from escrow to recipient
        let cpi_accounts = Transfer {
            from: self.escrow_token_account.to_account_info(),
            to: self.recipient_token_account.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
        
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(
            cpi_program.clone(),
            cpi_accounts,
            signer_seeds,
        );
        
        // Transfer all tokens from the escrow token account
        token::transfer(cpi_ctx, transfer_amount)?;
        
        // Close the escrow token account and return rent to initiator
        let cpi_accounts = CloseAccount {
            account: self.escrow_token_account.to_account_info(),
            destination: self.initiator.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new_with_signer(
            cpi_program,
            cpi_accounts,
            signer_seeds,
        );
        
        // Close the token account
        token::close_account(cpi_ctx)?;
        
        // Update registries
        self.initiator_registry.remove_escrow(self.escrow.key());
        self.beneficiary_registry.remove_escrow(self.escrow.key());
        
        Ok(())
    }
}

pub fn close_escrow(
    ctx: Context<CloseEscrow>,
    distribution_percentages: Vec<Distribution>,
) -> Result<()> {
    ctx.accounts.process(distribution_percentages)
} 