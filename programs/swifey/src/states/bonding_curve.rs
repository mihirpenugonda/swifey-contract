use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

use crate::errors::SwifeyError;
use crate::utils::{
    fixed_div_u128, fixed_mul_u128, sol_transfer_from_user, sol_transfer_with_signer, token_transfer_user, token_transfer_with_signer, CurveCompleted, TokenPurchased, TokenSold
};
use crate::utils::fixed_math::{fixed_mul, fixed_div, fixed_pow};
use crate::constants::{
    PRECISION, PRECISION_U64, CRR_NUMERATOR, CRR_DENOMINATOR, FEE_PRECISION,
    MIN_BUY_AMOUNT, MIN_SELL_AMOUNT, MAX_PRICE_IMPACT_BPS
};

// Minimum SOL liquidity threshold (1 SOL)
pub const MIN_SOL_LIQUIDITY: u64 = 1_000_000_000;  // 1 SOL in lamports

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
    pub fn update_reserves(&mut self, reserve_lamport: u64, reserve_token: u64) -> Result<bool> {
        // Check minimum SOL liquidity threshold
        require!(
            reserve_lamport >= MIN_SOL_LIQUIDITY,
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
        msg!("Calculating amount out preview - amount_in: {}, direction: {}, fee_percentage: {}", 
            amount_in, direction, fee_percentage);
        
        require!(self.virtual_sol_reserve > 0, SwifeyError::DivisionByZero);
        require!(self.virtual_token_reserve > 0, SwifeyError::DivisionByZero);

        // Check minimum amounts
        if direction == 0 {
            require!(amount_in >= MIN_BUY_AMOUNT, SwifeyError::DustAmount);
        }

        let amount_out = if direction == 0 { // Buying tokens
            msg!("Calculating buy amount...");
            
            // Calculate ratio = (new_sol/current_sol)^CRR
            let current_sol = self.virtual_sol_reserve;
            let new_sol = current_sol.checked_add(amount_in)
                .ok_or(SwifeyError::MathOverflow)?;
            
            msg!("Current SOL: {}, New SOL: {}", current_sol, new_sol);
            
            // Calculate ratio = (new_sol/current_sol)^CRR
            let ratio_base = fixed_div_u128(new_sol, current_sol)?;
            msg!("Ratio base: {}", ratio_base);
            
            let crr = fixed_div_u128(CRR_NUMERATOR, CRR_DENOMINATOR)?;
            msg!("CRR value: {}", crr);
            
            let ratio = fixed_pow(ratio_base as u64, crr as u64)?;
            msg!("Final ratio: {}", ratio);
            
            // Calculate tokens_out = total_tokens * (1 - 1/ratio)
            let tokens_out = (self.virtual_token_reserve as f64) * 
                (1.0 - (PRECISION as f64 / ratio as f64));
            
            tokens_out as u64
        } else { // Selling tokens
            // For selling, use the inverse formula
            let current_token = self.virtual_token_reserve;
            let new_token = current_token.checked_add(amount_in)
                .ok_or(SwifeyError::MathOverflow)?;
            
            let ratio_base = fixed_div_u128(new_token, current_token)?;
            let inverse_crr = fixed_div_u128(CRR_DENOMINATOR, CRR_NUMERATOR)?;
            let ratio = fixed_pow(ratio_base as u64, inverse_crr as u64)?;
            
            let sol_out = (self.virtual_sol_reserve as f64) * 
                (1.0 - (PRECISION as f64 / ratio as f64));
            
            sol_out as u64
        };

        // Calculate fee amount
        let fee_amount = if direction == 0 {
            msg!("Calculating buy fee...");
            (amount_in as f64 * fee_percentage as f64 / FEE_PRECISION as f64) as u64
        } else {
            (amount_out as f64 * fee_percentage as f64 / FEE_PRECISION as f64) as u64
        };

        Ok((amount_out, fee_amount))
    }

    // Calculate price impact as percentage in basis points (1% = 100 bps)
    pub fn calculate_price_impact(&self, amount_in: u64) -> Result<u64> {
        let current_price = fixed_div_u128(
            self.virtual_sol_reserve,
            self.virtual_token_reserve
        )?;
        msg!("Current price: {}", current_price);
        msg!("Current virtual SOL reserve: {}", self.virtual_sol_reserve);
        msg!("Current virtual token reserve: {}", self.virtual_token_reserve);
        
        let new_sol = self.virtual_sol_reserve.checked_add(amount_in)
            .ok_or_else(|| {
                msg!("Overflow in new_sol calculation: {} + {}", self.virtual_sol_reserve, amount_in);
                SwifeyError::MathOverflow
            })?;
        msg!("New SOL after amount_in: {}", new_sol);
            
        let (amount_out, _) = self.calculate_amount_out_preview(amount_in, 0, 0)?;
        msg!("Calculated amount_out: {}", amount_out);
        
        let new_token = self.virtual_token_reserve.checked_sub(amount_out)
            .ok_or_else(|| {
                msg!("Overflow in new_token calculation: {} - {}", self.virtual_token_reserve, amount_out);
                SwifeyError::MathOverflow
            })?;
        msg!("New token reserve: {}", new_token);
        
        let execution_price = fixed_div_u128(new_sol, new_token)?;
        msg!("Execution price: {}", execution_price);
        
        // Calculate price impact in basis points (1% = 100 bps)
        let price_diff = execution_price.checked_sub(current_price)
            .ok_or_else(|| {
                msg!("Overflow in price_diff calculation: {} - {}", execution_price, current_price);
                SwifeyError::MathOverflow
            })?;
        msg!("Price difference: {}", price_diff);
            
        let impact = fixed_div_u128(
            price_diff as u64,
            current_price as u64
        )?;
        msg!("Raw impact: {}", impact);
        
        let final_impact = ((impact * 10000) / PRECISION) as u64;
        msg!("Final impact in basis points: {} bps", final_impact);
        
        Ok(final_impact)
    }

    // Swap sol for tokens
    pub fn buy(
        &mut self,
        token_mint: &Account<'info, Mint>,
        curve_limit: u64,
        user: &Signer<'info>,
        curve_pda: &mut AccountInfo<'info>,
        fee_recipient: &mut AccountInfo<'info>,
        user_ata: &mut AccountInfo<'info>,
        curve_ata: &AccountInfo<'info>,
        amount_in: u64,
        min_amount_out: u64,
        fee_percentage: u64,
        curve_bump: u8,
        system_program: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
        max_price_impact: u64,
    ) -> Result<(u64, u64, u64, u64, u64, bool)> {
        msg!("Starting buy operation...");
        // Validate state before proceeding
        self.validate_state_transition()?;

        // Calculate and validate price impact
        let price_impact = self.calculate_price_impact(amount_in)?;
        msg!("Price impact: {} (max allowed: {})", price_impact, max_price_impact);
        require!(
            price_impact <= max_price_impact,
            SwifeyError::ExcessivePriceImpact
        );

        // Calculate amounts and fees
        let (amount_out, fee_amount) =
            self.calculate_amount_out_preview(amount_in, 0, fee_percentage)?;

        msg!("Calculated amounts - Out: {}, Fee: {}", amount_out, fee_amount);

        // Validate minimum output
        require!(
            amount_out >= min_amount_out,
            SwifeyError::InsufficientAmountOut
        );

        // Calculate new reserves
        let amount_in_after_fee = amount_in.checked_sub(fee_amount)
            .ok_or_else(|| {
                msg!("Overflow in amount_in_after_fee: {} - {}", amount_in, fee_amount);
                SwifeyError::MathOverflow
            })?;

        let new_sol_reserves = self.virtual_sol_reserve
            .checked_add(amount_in_after_fee)
            .ok_or_else(|| {
                msg!("Overflow in new_sol_reserves: {} + {}", self.virtual_sol_reserve, amount_in_after_fee);
                SwifeyError::MathOverflow
            })?;

        let new_token_reserves = self.virtual_token_reserve
            .checked_sub(amount_out)
            .ok_or_else(|| {
                msg!("Overflow in new_token_reserves: {} - {}", self.virtual_token_reserve, amount_out);
                SwifeyError::MathOverflow
            })?;

        msg!("New reserves calculated - SOL: {}, Token: {}", new_sol_reserves, new_token_reserves);

        // Perform transfers
        sol_transfer_from_user(&user, fee_recipient, system_program, fee_amount)?;
        sol_transfer_from_user(&user, curve_pda, system_program, amount_in_after_fee)?;
        token_transfer_with_signer(
            curve_ata,
            curve_pda,
            user_ata,
            token_program,
            &[&BondingCurve::get_signer(&token_mint.key(), &curve_bump)],
            amount_out,
        )?;

        // Update reserves
        self.update_reserves(new_sol_reserves, new_token_reserves)?;

        // Check if curve is completed
        let is_completed = self.update_completion_state(new_sol_reserves, curve_limit)?;

        Ok((amount_in, amount_out, fee_amount, new_sol_reserves, new_token_reserves, is_completed))
    }

    // Swap tokens for sol
    pub fn sell(
        &mut self,
        token_mint: &Account<'info, Mint>,
        user: &Signer<'info>,
        curve_pda: &mut AccountInfo<'info>,
        user_ata: &mut AccountInfo<'info>,
        fee_recipient: &mut AccountInfo<'info>,
        curve_ata: &mut AccountInfo<'info>,
        amount_in: u64,
        min_amount_out: u64,
        fee_percentage: u64,
        curve_bump: u8,
        system_program: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
        curve_limit: u64,
    ) -> Result<(u64, u64, u64, u64)> {
        // Validate state before proceeding
        self.validate_state_transition()?;

        // 1. Calculate amounts and fees
        let (amount_out, fee_amount) =
            self.calculate_amount_out_preview(amount_in, 1, fee_percentage)?;
        
        // 2. Validate amounts
        require!(
            amount_out >= min_amount_out,
            SwifeyError::InsufficientAmountOut
        );

        // 3. Calculate new reserves with checked arithmetic
        let new_token_reserves = self.virtual_token_reserve
            .checked_add(amount_in)
            .ok_or(SwifeyError::MathOverflow)?;

        let amount_out_with_fee = amount_out.checked_add(fee_amount)
            .ok_or(SwifeyError::MathOverflow)?;

        let new_sol_reserves = self.virtual_sol_reserve
            .checked_sub(amount_out_with_fee)
            .ok_or(SwifeyError::MathOverflow)?;

        // 4. Validate SOL balance and minimum liquidity
        let pda_sol_balance = curve_pda.lamports();
        require!(
            pda_sol_balance >= amount_out_with_fee,
            SwifeyError::InsufficientSolBalance
        );

        // 5. Perform transfers first
        let token = token_mint.key();
        let signer_seeds: &[&[&[u8]]] = &[&BondingCurve::get_signer(&token, &curve_bump)];

        token_transfer_user(user_ata, curve_ata, user, token_program, amount_in)?;
        sol_transfer_with_signer(
            curve_pda,
            user,      
            system_program,
            signer_seeds,
            amount_out,
        )?;
        sol_transfer_with_signer(
            curve_pda, 
            fee_recipient,
            system_program,
            signer_seeds,
            fee_amount,
        )?;

        // 6. Update reserves only after all transfers succeed
        self.update_reserves(new_sol_reserves, new_token_reserves)?;

        let price = new_sol_reserves
            .checked_div(new_token_reserves)
            .ok_or(SwifeyError::DivisionByZero)?;

        Ok((amount_in, amount_out, fee_amount, price))
    }
}
