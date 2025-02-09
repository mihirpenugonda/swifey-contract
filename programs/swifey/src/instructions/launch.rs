use crate::{
    constants::TOKEN_DECIMAL, errors::SwifeyError, states::{BondingCurve, Config}, constants::{LAMPORTS_PER_SOL},  utils::{TokenLaunched}
};

use anchor_lang::{prelude::*, system_program, solana_program::sysvar};

use anchor_spl::{
    associated_token::{self, AssociatedToken},
    metadata::{self, mpl_token_metadata::types::DataV2, Metadata},
    token::{self, spl_token::instruction::AuthorityType, Mint, Token, TokenAccount}
};

pub fn launch(
    ctx: Context<Launch>,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {
    // Check if contract is paused
    require!(!ctx.accounts.global_config.is_paused, SwifeyError::ContractPaused);

    let rent = Rent::get()?;
    let min_rent = rent.minimum_balance(ctx.accounts.bonding_curve.to_account_info().data_len());
    
    // Add 2x rent buffer for safety
    let rent_buffer = min_rent.checked_mul(2)
        .ok_or(SwifeyError::MathOverflow)?;

    msg!("Adding rent buffer of {} lamports to bonding curve PDA", rent_buffer);

    // Transfer additional rent buffer to the bonding curve PDA
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.creator.to_account_info(),
                to: ctx.accounts.bonding_curve.to_account_info(),
            },
        ),
        rent_buffer,
    )?;

    let bonding_curve = &mut ctx.accounts.bonding_curve;
    let global_config = &ctx.accounts.global_config;

    // Validate initial parameters
    // 1. Validate virtual token reserve is at least 80% of total supply
    require!(
        global_config.initial_virtual_token_reserve >= (global_config.total_token_supply * 80) / 100,
        SwifeyError::InvalidTokenAllocation
    );

    // 2. Validate initial SOL reserve is non-zero and reasonable
    require!(
        global_config.initial_virtual_sol_reserve >= LAMPORTS_PER_SOL, // At least 1 SOL
        SwifeyError::InsufficientLiquidity
    );

    // 3. Validate curve limit is greater than initial SOL reserve
    require!(
        global_config.curve_limit > global_config.initial_virtual_sol_reserve,
        SwifeyError::InvalidCurveLimit
    );

    // 4. Validate total token supply is non-zero
    require!(
        global_config.total_token_supply > 0,
        SwifeyError::InvalidTokenAllocation
    );

    // 5. Validate initial real token reserve is zero or less than virtual reserve
    require!(
        global_config.initial_real_token_reserve <= global_config.initial_virtual_token_reserve,
        SwifeyError::InvalidTokenAllocation
    );

    // init bonding curve pda with initial SOL balance including rent buffer
    bonding_curve.virtual_token_reserve = global_config.initial_virtual_token_reserve;
    bonding_curve.virtual_sol_reserve = global_config.initial_virtual_sol_reserve;
    bonding_curve.real_token_reserve = global_config.initial_real_token_reserve;
    bonding_curve.real_sol_reserve = rent_buffer; // Initialize with rent buffer
    bonding_curve.token_total_supply = global_config.total_token_supply;
    bonding_curve.is_completed = false;
    bonding_curve.is_migrated = false;

    let signer_seeds: &[&[&[u8]]] = &[&[Config::SEED_PREFIX.as_bytes(), &[ctx.bumps.global_config]]];

    //  mint token to bonding curve
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.curve_token_account.to_account_info(),
                authority: global_config.to_account_info(),
            },
            signer_seeds,
        ),
        global_config.total_token_supply,
    )?;

    //  create metadata
    metadata::create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.metadata_program.to_account_info(),
            metadata::CreateMetadataAccountsV3 {
                metadata: ctx.accounts.token_metadata_account.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
                mint_authority: global_config.to_account_info(),
                payer: ctx.accounts.creator.to_account_info(),
                update_authority: global_config.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            signer_seeds,
        ),
        DataV2 {
            name: name.clone(),
            symbol: symbol.clone(),
            uri: uri.clone(),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        },
        false,
        true,
        None,
    )?;

    //  revoke mint authority
    token::set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::SetAuthority {
                current_authority: global_config.to_account_info(),
                account_or_mint: ctx.accounts.token_mint.to_account_info(),
            },
            signer_seeds,
        ),
        AuthorityType::MintTokens,
        None,
    )?;

    emit_cpi!(TokenLaunched {
        token_mint: ctx.accounts.token_mint.key(),
        name: name,
        symbol: symbol,
        uri: uri,
        creator: ctx.accounts.creator.key(),
        initial_virtual_sol_reserve: global_config.initial_virtual_sol_reserve,
        initial_virtual_token_reserve: global_config.initial_virtual_token_reserve,
        total_token_supply: global_config.total_token_supply,
        curve_limit: global_config.curve_limit
    });

    Ok(())
}

#[derive(Accounts)]
#[event_cpi]
pub struct Launch<'info>{
    #[account(mut)]
    creator: Signer<'info>,

    #[account(
        seeds = [Config::SEED_PREFIX.as_bytes()],
        bump
    )]
    global_config: Box<Account<'info, Config>>,

    #[account(
        init, 
        payer = creator,
        mint::decimals = TOKEN_DECIMAL,
        mint::authority = global_config.key(),
    )]
    token_mint: Box<Account<'info, Mint>>,

    #[account(
        init, 
        payer = creator,
        space = 8 + BondingCurve::LEN,
        seeds = [BondingCurve::SEED_PREFIX.as_bytes(), &token_mint.key().to_bytes()],
        bump
    )]
    bonding_curve: Box<Account<'info, BondingCurve>>,

    #[account(
        init, 
        payer = creator,
        associated_token::mint = token_mint,
        associated_token::authority = bonding_curve,
    )]
    curve_token_account: Box<Account<'info, TokenAccount>>,
    
    /// CHECK: Initialized by token metadata program
    #[account(mut)]
    token_metadata_account: UncheckedAccount<'info>,

    #[account(address = token::ID)]
    token_program: Program<'info, Token>,

    #[account(address = associated_token::ID)]
    associated_token_program: Program<'info, AssociatedToken>,

    #[account(address = metadata::ID)]
    metadata_program: Program<'info, Metadata>,

    #[account(address = system_program::ID)]
    system_program: Program<'info, System>,

    #[account(address = sysvar::rent::ID)]
    rent: Sysvar<'info, Rent>,
}