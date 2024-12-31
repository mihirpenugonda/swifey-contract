use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use raydium_amm_v3::state::AMM;

use crate::states::{BondingCurve, Config};
use crate::errors::SwifeyError;
use crate::utils::{token_transfer_with_signer, sol_transfer_with_signer};


#[derive(Accounts)]
pub struct Migrate<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [BondingCurve::SEED_PREFIX.as_bytes(), token_mint.key().as_ref()],
        bump,
        constraint = bonding_curve.is_completed @ SwifeyError::CurveNotCompleted,
        constraint = !bonding_curve.is_migrated @ SwifeyError::AlreadyMigrated,
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    #[account(mut)]
    pub token_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = curve_token_account.owner == bonding_curve.key(),
        constraint = curve_token_account.amount > 0 @ SwifeyError::InsufficientTokenBalance,
    )]
    pub curve_token_account: Account<'info, TokenAccount>,

    /// CHECK: Native SOL account owned by bonding curve
    #[account(
        mut,
        constraint = curve_sol_account.owner == bonding_curve.key(),
        constraint = curve_sol_account.lamports() > 0 @ SwifeyError::InsufficientSolBalance,
    )]
    pub curve_sol_account: AccountInfo<'info>,

    #[account(
        mut,
        constraint = raydium_pool.owner == &raydium_amm_v3::ID @ SwifeyError::InvalidPoolOwner,
    )]
    pub raydium_pool: Account<'info, AMM>,

    /// CHECK: Fee recipient account
    #[account(mut)]
    pub fee_recipient: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct MigrateParams {
    minimum_sol_amount: u64,
    minimum_token_amount: u64,
}

impl<'info> Migrate<'info> {
    pub fn process(ctx: Context<Migrate>, params: MigrateParams) -> Result<()> {
        let bonding_curve = &mut ctx.accounts.bonding_curve;
        let config = &ctx.accounts.config;
        let raydium_pool = &ctx.accounts.raydium_pool;
        
        // Verify authority
        require!(
            ctx.accounts.authority.key() == config.authority,
            SwifeyError::UnauthorizedAddress
        );

        // Verify Raydium pool state
        require!(
            raydium_pool.state == raydium_amm_v3::state::PoolState::Initialized,
            SwifeyError::InvalidPoolState
        );

        // Verify pool tokens match
        require!(
            raydium_pool.token_mint_0 == ctx.accounts.token_mint.key() || 
            raydium_pool.token_mint_1 == ctx.accounts.token_mint.key(),
            SwifeyError::InvalidPoolTokens
        );
    
        // Get signer seeds for PDA operations
        let seeds = BondingCurve::get_signer(
            &ctx.accounts.token_mint.key(),
            &ctx.bumps.bonding_curve
        );
        let signer_seeds = &[&seeds[..]];
    
        // Calculate migration fee using integer math to avoid precision loss
        let sol_balance = ctx.accounts.curve_sol_account.lamports();
        let migration_fee = sol_balance
            .checked_mul(config.migration_fee_percentage as u64)
            .ok_or(SwifeyError::MathOverflow)?
            .checked_div(100)
            .ok_or(SwifeyError::MathOverflow)?;
    
        // Calculate remaining amounts
        let remaining_sol = sol_balance
            .checked_sub(migration_fee)
            .ok_or(SwifeyError::InsufficientSolBalance)?;
        let token_balance = ctx.accounts.curve_token_account.amount;

        // Verify minimum amounts
        require!(
            remaining_sol >= params.minimum_sol_amount,
            SwifeyError::SlippageExceeded
        );
        require!(
            token_balance >= params.minimum_token_amount,
            SwifeyError::SlippageExceeded
        );
    
        // Transfer migration fee to fee recipient
        sol_transfer_with_signer(
            &ctx.accounts.curve_sol_account.to_account_info(),
            &ctx.accounts.fee_recipient,
            &ctx.accounts.system_program,
            signer_seeds,
            migration_fee,
        )?;
    
        // Transfer remaining SOL to Raydium pool
        sol_transfer_with_signer(
            &ctx.accounts.curve_sol_account.to_account_info(),
            &ctx.accounts.raydium_pool.to_account_info(),
            &ctx.accounts.system_program,
            signer_seeds,
            remaining_sol,
        )?;
    
        // Transfer tokens to Raydium pool
        token_transfer_with_signer(
            &ctx.accounts.curve_token_account.to_account_info(),
            &bonding_curve.to_account_info(),
            &ctx.accounts.raydium_pool.to_account_info(),
            &ctx.accounts.token_program.to_account_info(),
            signer_seeds,
            token_balance,
        )?;
    
        // Mark as migrated
        bonding_curve.is_migrated = true;
    
        msg!("Migration completed successfully");
        msg!("Migrated SOL amount: {}", remaining_sol);
        msg!("Migrated token amount: {}", token_balance);
        msg!("Migration fee paid: {}", migration_fee);
        
        Ok(())
    }
}