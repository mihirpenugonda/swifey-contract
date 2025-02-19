use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

use crate::errors::SwifeyError;
use crate::utils::{
    fixed_div_u128, fixed_mul_u128, sol_transfer_from_user, sol_transfer_with_signer, token_transfer_user, token_transfer_with_signer, CurveCompleted, TokenPurchased, TokenSold
};
use crate::constants::{
    PRECISION, CRR_NUMERATOR, CRR_DENOMINATOR,
    MIN_BUY_AMOUNT, FEE_PRECISION
};
use crate::states::Config;

// Minimum SOL liquidity threshold (1 SOL)
pub const MIN_SOL_LIQUIDITY: u64 = 1_000_000_000;  // 1 SOL in lamports

// For reference:
// - Total supply: 1,000,000,000.000000 tokens (1B with 6 decimals)
// - Initial price: ~0.000005 SOL per token (5 SOL / 1B tokens)
// - Initial SOL reserve: 12.5 SOL
// - Target SOL limit: 100 SOL
// - CRR: 0.651 (defined in constants.rs)
// - All tokens start in virtual reserve for proper price discovery

#[account]
pub struct BondingCurve {
    //Virtual reserves on the curve
    pub virtual_token_reserve: u64,
    pub virtual_sol_reserve: u64,

    //Real reserves on the curve
    pub real_token_reserve: u64,
    pub real_sol_reserve: u64,

    //Token total supply
    pub token_total_supply: u64,

    //Is the curve completed
    pub is_completed: bool,

    // New field to track if funds are migrated to Raydium
    pub is_migrated: bool,

    // Reserved field for padding
    pub reserved: [u8; 8]
}

impl<'info> BondingCurve {
    pub const SEED_PREFIX: &'static str = "bonding_curve";
    pub const LEN: usize = 8 * 5 + 1 + 1 + 8;  // Back to original size

    //Get signer for bonding curve PDA
    pub fn get_signer<'a>(mint: &'a Pubkey, bump: &'a u8) -> [&'a [u8]; 3] {
        [
            Self::SEED_PREFIX.as_bytes(),
            mint.as_ref(),
            std::slice::from_ref(bump),
        ]
    }

    //Update reserves with minimum liquidity check
    pub fn update_reserves(&mut self, reserve_lamport: u64, reserve_token: u64, initial_sol_reserve: u64) -> Result<bool> {
        // Check minimum SOL liquidity threshold
        require!(
            reserve_lamport >= initial_sol_reserve,
            SwifeyError::InsufficientLiquidity
        );

        self.virtual_sol_reserve = reserve_lamport;
        self.virtual_token_reserve = reserve_token;

        Ok(true)
    }

    // Helper to validate state transitions
    pub fn validate_state_transition(&self) -> Result<()> {
        // Prevent operations if already migrated
        require!(!self.is_migrated, SwifeyError::AlreadyMigrated);
        Ok(())
    }

    // Helper to safely update completion state
    pub fn update_completion_state(&mut self, new_sol_reserves: u64, curve_limit: u64) -> Result<bool> {
        let is_completed = if new_sol_reserves >= curve_limit {
            self.is_completed = true;
            true
        } else {
            false
        };
        Ok(is_completed)
    }

    // Helper to safely update migration state
    pub fn update_migration_state(&mut self) -> Result<()> {
        require!(self.is_completed, SwifeyError::CurveNotCompleted);
        require!(!self.is_migrated, SwifeyError::AlreadyMigrated);
        
        self.is_migrated = true;
        Ok(())
    }

    // Calculate amount out without modifying state
    pub fn calculate_amount_out_preview(&self, amount_in: u64, direction: u8, fee_percentage: u64) -> Result<(u64, u64)> {
        require!(self.virtual_sol_reserve > 0, SwifeyError::DivisionByZero);
        require!(self.virtual_token_reserve > 0, SwifeyError::DivisionByZero);

        // Check minimum amounts
        if direction == 0 {
            require!(amount_in >= MIN_BUY_AMOUNT, SwifeyError::DustAmount);
        }

        let mut amount_out = if direction == 0 { // Buying tokens
            // Calculate using CRR formula: tokens_out = total_tokens * (1 - (current_sol/new_sol)^CRR)
            let current_sol = self.virtual_sol_reserve;
            let new_sol = current_sol.checked_add(amount_in)
                .ok_or(SwifeyError::MathOverflow)?;
            
            // Calculate ratio = current_sol/new_sol (scaled by PRECISION)
            let ratio = fixed_div_u128(current_sol, new_sol)?;
            
            // Calculate CRR (as a fraction of PRECISION)
            let crr = fixed_div_u128(CRR_NUMERATOR, CRR_DENOMINATOR)?;
            
            // Approximate the power function for values close to 1 using this formula:
            // (1 + x)^a ≈ 1 + ax for x close to 0, where x = ratio - 1
            // In our case, ratio is current_sol/new_sol which is less than 1
            // So we use: ratio^CRR ≈ 1 - CRR*(1 - ratio)
            let one_minus_ratio = PRECISION.checked_sub(ratio)
                .ok_or(SwifeyError::MathOverflow)?;
            
            let crr_effect = (crr as u128)
                .checked_mul(one_minus_ratio)
                .ok_or(SwifeyError::MathOverflow)?
                .checked_div(PRECISION)
                .ok_or(SwifeyError::DivisionByZero)?;
            
            let final_ratio = PRECISION.checked_sub(crr_effect)
                .ok_or(SwifeyError::MathOverflow)?;
            
            // Calculate tokens_out = total_tokens * (1 - final_ratio)
            let tokens_out = (self.virtual_token_reserve as u128)
                .checked_mul(PRECISION.checked_sub(final_ratio)
                    .ok_or(SwifeyError::MathOverflow)?)
                .ok_or(SwifeyError::MathOverflow)?
                .checked_div(PRECISION)
                .ok_or(SwifeyError::DivisionByZero)? as u64;
            
            tokens_out
        } else { // Selling tokens
            // For selling, use similar approximation but in reverse
            let current_token = self.virtual_token_reserve;
            let new_token = current_token.checked_add(amount_in)
                .ok_or(SwifeyError::MathOverflow)?;
            
            // Calculate ratio = current_token/new_token (scaled by PRECISION)
            let ratio = fixed_div_u128(current_token, new_token)?;
            
            // Calculate inverse CRR
            let inverse_crr = fixed_div_u128(CRR_DENOMINATOR, CRR_NUMERATOR)?;
            
            // Use the same approximation
            let one_minus_ratio = PRECISION.checked_sub(ratio)
                .ok_or(SwifeyError::MathOverflow)?;
            
            let crr_effect = (inverse_crr as u128)
                .checked_mul(one_minus_ratio)
                .ok_or(SwifeyError::MathOverflow)?
                .checked_div(PRECISION)
                .ok_or(SwifeyError::DivisionByZero)?;
            
            let final_ratio = PRECISION.checked_sub(crr_effect)
                .ok_or(SwifeyError::MathOverflow)?;
            
            // Calculate base sol_out = total_sol * (1 - final_ratio)
            let base_sol_out = (self.virtual_sol_reserve as u128)
                .checked_mul(PRECISION.checked_sub(final_ratio)
                    .ok_or(SwifeyError::MathOverflow)?)
                .ok_or(SwifeyError::MathOverflow)?
                .checked_div(PRECISION)
                .ok_or(SwifeyError::DivisionByZero)? as u64;

            // Calculate total amount needed including fee
            let total_sol_out = base_sol_out;
            
            total_sol_out
        };

        // Calculate fee amount using fixed-point arithmetic
        let fee_amount = if direction == 0 {
            (amount_in as u128)
                .checked_mul(fee_percentage as u128)
                .ok_or(SwifeyError::MathOverflow)?
                .checked_div(FEE_PRECISION as u128)
                .ok_or(SwifeyError::DivisionByZero)? as u64
        } else {
            (amount_out as u128)
                .checked_mul(fee_percentage as u128)
                .ok_or(SwifeyError::MathOverflow)?
                .checked_div(FEE_PRECISION as u128)
                .ok_or(SwifeyError::DivisionByZero)? as u64
        };

        Ok((amount_out, fee_amount))
    }

    // Swap sol for tokens
    pub fn buy(
        &mut self,
        token_mint: &Account<'info, Mint>,
        config: &Account<'info, Config>,
        user: &Signer<'info>,
        curve_pda: &mut AccountInfo<'info>,
        fee_recipient: &mut AccountInfo<'info>,
        user_ata: &mut AccountInfo<'info>,
        curve_ata: &AccountInfo<'info>,
        amount_in: u64,
        min_amount_out: u64,
        curve_bump: u8,
        system_program: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
    ) -> Result<(u64, u64, u64, u64, u64, bool)> {
        // Validate state before proceeding
        self.validate_state_transition()?;

        // Calculate fee to be added on top of amount_in
        let fee_amount = if config.buy_fee_percentage > 0 {
            (amount_in as u128)
                .checked_mul(config.buy_fee_percentage as u128)
                .ok_or(SwifeyError::MathOverflow)?
                .checked_div(FEE_PRECISION as u128)
                .ok_or(SwifeyError::DivisionByZero)? as u64
        } else {
            0
        };
        
        // Total amount user will pay is amount_in + fee_amount
        let total_amount_in = amount_in.checked_add(fee_amount)
            .ok_or(SwifeyError::MathOverflow)?;

        // Validate user has enough SOL to cover amount_in + fees
        let user_balance = user.lamports();
        require!(
            user_balance >= total_amount_in,
            SwifeyError::InsufficientUserBalance
        );

        // Calculate amounts using the original amount_in (not including fees)
        let (amount_out, _) = self.calculate_amount_out_preview(amount_in, 0, 0)?;

        // Validate minimum output
        require!(
            amount_out >= min_amount_out,
            SwifeyError::InsufficientAmountOut
        );

        // Calculate new reserves using original amount_in
        let new_sol_reserves = self.virtual_sol_reserve
            .checked_add(amount_in)
            .ok_or(SwifeyError::MathOverflow)?;

        let new_token_reserves = self.virtual_token_reserve
            .checked_sub(amount_out)
            .ok_or(SwifeyError::MathOverflow)?;

        // Perform transfers - user pays total_amount_in (amount_in + fee_amount)
        sol_transfer_from_user(&user, fee_recipient, system_program, fee_amount)?;
        sol_transfer_from_user(&user, curve_pda, system_program, amount_in)?;
        token_transfer_with_signer(
            curve_ata,
            curve_pda,
            user_ata,
            token_program,
            &[&BondingCurve::get_signer(&token_mint.key(), &curve_bump)],
            amount_out,
        )?;

        // Update reserves
        self.update_reserves(new_sol_reserves, new_token_reserves, config.initial_virtual_sol_reserve)?;

        // Check if curve is completed
        let is_completed = self.update_completion_state(new_sol_reserves, config.curve_limit)?;

        if is_completed {
            emit!(CurveCompleted {
                token_mint: token_mint.key(),
                final_sol_reserve: new_sol_reserves,
                final_token_reserve: new_token_reserves,
            });
        }

        emit!(TokenPurchased {
            token_mint: token_mint.key(),
            buyer: user.key(),
            sol_amount: amount_in,
            token_amount: amount_out,
            fee_amount: fee_amount,
            price: new_sol_reserves / new_token_reserves,
            new_sol_reserves,
            new_token_reserves
        });

        Ok((amount_in, amount_out, fee_amount, new_sol_reserves, new_token_reserves, is_completed))
    }

    // Swap tokens for sol
    pub fn sell(
        &mut self,
        token_mint: &Account<'info, Mint>,
        config: &Account<'info, Config>,
        user: &Signer<'info>,
        curve_pda: &mut AccountInfo<'info>,
        user_ata: &mut AccountInfo<'info>,
        fee_recipient: &mut AccountInfo<'info>,
        curve_ata: &mut AccountInfo<'info>,
        amount_in: u64,
        min_amount_out: u64,
        curve_bump: u8,
        system_program: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
    ) -> Result<(u64, u64, u64, u64, u64)> {
        // Validate state before proceeding
        self.validate_state_transition()?;

        // Calculate amounts and fees
        let (amount_out, fee_amount) =
            self.calculate_amount_out_preview(amount_in, 1, config.sell_fee_percentage)?;

        // Calculate new reserves
        let new_token_reserves = self.virtual_token_reserve
            .checked_add(amount_in)
            .ok_or(SwifeyError::MathOverflow)?;

        let new_sol_reserves = self.virtual_sol_reserve
            .checked_sub(amount_out)
            .ok_or(SwifeyError::MathOverflow)?;

        // Calculate user amount (amount minus fees)
        let user_amount = amount_out.checked_sub(fee_amount)
            .ok_or(SwifeyError::MathOverflow)?;

        // Validate SOL balance - check if we have enough for both user amount and fees
        let pda_sol_balance = curve_pda.lamports();
        require!(pda_sol_balance >= user_amount, SwifeyError::InsufficientSolBalance);

        let token = token_mint.key();
        let signer_seeds: &[&[&[u8]]] = &[&BondingCurve::get_signer(&token, &curve_bump)];

        // First transfer tokens from user to curve
        token_transfer_user(user_ata, curve_ata, user, token_program, amount_in)?;
        
        // Transfer the amount minus fees to user
        sol_transfer_with_signer(
            curve_pda,
            user,      
            system_program,
            signer_seeds,
            user_amount,
        )?;

        // Transfer fees to fee recipient if there are any and if we have enough balance
        if fee_amount > 0 && pda_sol_balance.checked_sub(user_amount).unwrap_or(0) >= fee_amount {
            sol_transfer_with_signer(
                curve_pda, 
                fee_recipient,
                system_program,
                signer_seeds,
                fee_amount,
            )?;
        }

        // Update reserves
        self.update_reserves(new_sol_reserves, new_token_reserves, config.initial_virtual_sol_reserve)?;

        emit!(TokenSold {
            token_mint: token_mint.key(),
            buyer: user.key(),
            sol_amount: user_amount,
            token_amount: amount_in,
            fee_amount,
            price: new_sol_reserves / new_token_reserves,
            new_sol_reserves,
            new_token_reserves
        });

        Ok((amount_in, amount_out, fee_amount, new_sol_reserves, new_token_reserves))
    }
}
