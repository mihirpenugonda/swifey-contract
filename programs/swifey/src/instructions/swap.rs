use crate::{
    errors::SwifeyError,
    states::{BondingCurve, Config}, utils::{CurveCompleted, TokenPurchased, TokenSold},
};

use anchor_lang::{prelude::*, system_program};

use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};

pub fn swap(ctx: Context<Swap>, amount: u64, direction: u8, min_out: u64) -> Result<()> {
    let bonding_curve = &mut ctx.accounts.bonding_curve;
    
    require!(bonding_curve.is_completed == false, SwifeyError::CurveLimitReached);
    
    require!(direction == 0 || direction == 1, SwifeyError::InvalidDirection);

    // Add minimum amount check (0.001 SOL = 1_000_000 lamports)
    if direction == 0 {
        require!(amount >= 1_000_000, SwifeyError::DustAmount);
    } else {
        require!(amount >= 1000, SwifeyError::DustAmount);
    }

    let curve_pda = &mut bonding_curve.to_account_info();
    let global_config = &ctx.accounts.global_config;

    if direction == 0 {
        let (amount_in, amount_out, fee_amount, new_sol_reserves, new_token_reserves, is_completed) = bonding_curve.buy(
            &ctx.accounts.token_mint,
            global_config.curve_limit,
            &ctx.accounts.user,
            curve_pda,
            &mut ctx.accounts.fee_recipient,
            &mut ctx.accounts.user_token_account.to_account_info(),
            &mut ctx.accounts.curve_token_account.to_account_info(),
            amount,
            min_out,
            global_config.buy_fee_percentage,
            ctx.bumps.bonding_curve,
            &ctx.accounts.system_program.to_account_info(),
            &ctx.accounts.token_program.to_account_info()
        )?;

        if is_completed {
            emit_cpi!(CurveCompleted {
                token_mint: ctx.accounts.token_mint.key(),
                final_sol_reserve: new_sol_reserves,
                final_token_reserve: new_token_reserves,
            });
        }

        emit_cpi!(TokenPurchased {
            token_mint: ctx.accounts.token_mint.key(),
            buyer: ctx.accounts.user.key(),
            sol_amount: amount_in,
            token_amount: amount_out,
            fee_amount: fee_amount,
            price: new_sol_reserves / new_token_reserves
        });
    } else if direction == 1 {
        let (amount_in, amount_out, fee_amount, price) = bonding_curve.sell(
            &ctx.accounts.token_mint,
            &ctx.accounts.user,
            curve_pda,
            &mut ctx.accounts.user_token_account.to_account_info(),
            &mut ctx.accounts.fee_recipient,
            &mut ctx.accounts.curve_token_account.to_account_info(),
            amount,
            min_out,
            global_config.sell_fee_percentage,
            ctx.bumps.bonding_curve,
            &ctx.accounts.system_program.to_account_info(),
            &ctx.accounts.token_program.to_account_info()
        )?;

        emit_cpi!(TokenSold {
            token_mint: ctx.accounts.token_mint.key(),
            sol_amount: amount_in,
            token_amount: amount_out,
            fee_amount: fee_amount,
            price: price
        });
    }
    Ok(())
}

#[event_cpi]
#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    user: Signer<'info>,

    #[account(seeds = [Config::SEED_PREFIX.as_bytes()], bump)]
    global_config: Box<Account<'info, Config>>,

    /// CHECK: This account is verified by through the global config constraint
    #[account(mut, constraint = global_config.fee_recipient == fee_recipient.key() @SwifeyError::IncorrectFeeRecipient)]
    fee_recipient: AccountInfo<'info>,

    #[account(mut, seeds = [BondingCurve::SEED_PREFIX.as_bytes(), &token_mint.key().to_bytes()], bump)]
    bonding_curve: Box<Account<'info, BondingCurve>>,

    token_mint: Box<Account<'info, Mint>>,

    #[account(mut, associated_token::mint = token_mint, associated_token::authority = bonding_curve)]
    curve_token_account: Box<Account<'info, TokenAccount>>,

    #[account(init_if_needed, payer = user, associated_token::mint = token_mint, associated_token::authority = user)]
    user_token_account: Box<Account<'info, TokenAccount>>,

    #[account(address = token::ID)]
    token_program: Program<'info, Token>,
    #[account(address = associated_token::ID)]
    associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = system_program::ID)]
    system_program: Program<'info, System>,
}