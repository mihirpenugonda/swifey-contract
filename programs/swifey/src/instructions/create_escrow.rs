use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use anchor_spl::associated_token::AssociatedToken;
use crate::errors::SwifeyError;
use crate::states::*;

#[derive(Accounts)]
#[instruction(amount: u64, expires_at: Option<i64>)]
pub struct CreateEscrow<'info> {
    #[account(mut)]
    pub initiator: Signer<'info>,
    
    /// CHECK: This is the beneficiary of the escrow
    pub beneficiary: UncheckedAccount<'info>,
    
    #[account(
        mut,
        constraint = initiator_token_account.owner == initiator.key() @ SwifeyError::UnauthorizedAddress,
        constraint = initiator_token_account.mint == mint.key() @ SwifeyError::InvalidPoolTokens,
        constraint = initiator_token_account.amount >= amount @ SwifeyError::InsufficientTokenBalance,
    )]
    pub initiator_token_account: Account<'info, TokenAccount>,
    
    /// The mint of the token being escrowed
    pub mint: Account<'info, anchor_spl::token::Mint>,
    
    #[account(
        init,
        payer = initiator,
        space = Escrow::LEN,
        seeds = [
            ESCROW_SEED,
            initiator.key().as_ref(),
            beneficiary.key().as_ref(),
            mint.key().as_ref(),
        ],
        bump
    )]
    pub escrow: Account<'info, Escrow>,
    
    #[account(
        init_if_needed,
        payer = initiator,
        associated_token::mint = mint,
        associated_token::authority = escrow,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = initiator,
        space = EscrowRegistry::LEN,
        seeds = [
            ESCROW_REGISTRY_SEED,
            initiator.key().as_ref(),
        ],
        bump
    )]
    pub initiator_registry: Account<'info, EscrowRegistry>,
    
    #[account(
        init_if_needed,
        payer = initiator,
        space = EscrowRegistry::LEN,
        seeds = [
            ESCROW_REGISTRY_SEED,
            beneficiary.key().as_ref(),
        ],
        bump
    )]
    pub beneficiary_registry: Account<'info, EscrowRegistry>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> CreateEscrow<'info> {
    pub fn process(
        &mut self,
        amount: u64,
        expires_at: Option<i64>,
        escrow_bump: u8,
        initiator_registry_bump: u8,
        beneficiary_registry_bump: u8,
    ) -> Result<()> {
        // Transfer tokens from initiator to escrow account
        let cpi_accounts = Transfer {
            from: self.initiator_token_account.to_account_info(),
            to: self.escrow_token_account.to_account_info(),
            authority: self.initiator.to_account_info(),
        };
        
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::transfer(cpi_ctx, amount)?;
        
        // Initialize escrow account data
        let escrow = &mut self.escrow;
        escrow.initiator = self.initiator.key();
        escrow.beneficiary = self.beneficiary.key();
        escrow.escrow_token_account = self.escrow_token_account.key();
        escrow.mint = self.mint.key();
        escrow.amount = amount;
        escrow.created_at = Clock::get()?.unix_timestamp;
        escrow.expires_at = expires_at;
        escrow.is_closed = false;
        escrow.bump = escrow_bump;
        
        // Initialize initiator registry if it's new
        let initiator_registry = &mut self.initiator_registry;
        if initiator_registry.user == Pubkey::default() {
            initiator_registry.user = self.initiator.key();
            initiator_registry.initiated_escrows = Vec::new();
            initiator_registry.beneficiary_escrows = Vec::new();
            initiator_registry.bump = initiator_registry_bump;
        }
        
        // Initialize beneficiary registry if it's new
        let beneficiary_registry = &mut self.beneficiary_registry;
        if beneficiary_registry.user == Pubkey::default() {
            beneficiary_registry.user = self.beneficiary.key();
            beneficiary_registry.initiated_escrows = Vec::new();
            beneficiary_registry.beneficiary_escrows = Vec::new();
            beneficiary_registry.bump = beneficiary_registry_bump;
        }
        
        // Add escrow to registries
        initiator_registry.add_initiated_escrow(escrow.key())?;
        beneficiary_registry.add_beneficiary_escrow(escrow.key())?;
        
        Ok(())
    }
}

pub fn create_escrow(
    ctx: Context<CreateEscrow>,
    amount: u64,
    expires_at: Option<i64>,
) -> Result<()> {
    let escrow_bump = ctx.bumps.escrow;
    let initiator_registry_bump = ctx.bumps.initiator_registry;
    let beneficiary_registry_bump = ctx.bumps.beneficiary_registry;
    
    ctx.accounts.process(
        amount, 
        expires_at, 
        escrow_bump, 
        initiator_registry_bump, 
        beneficiary_registry_bump
    )
} 