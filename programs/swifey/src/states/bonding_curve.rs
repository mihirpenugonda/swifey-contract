use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::errors::SwifeyError;
use crate::utils::{
    sol_transfer_from_user, sol_transfer_with_signer, token_transfer_user,
    token_transfer_with_signer, TokenPurchased, TokenSold, CurveCompleted
};
use crate::utils::fixed_math::{fixed_mul, fixed_div, fixed_pow};
use crate::constants::{PRECISION_U64, CRR_NUMERATOR, FEE_PRECISION};

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
    pub const LEN: usize = 8 * 5 + 1 + 1 + 8;

    //Get signer for bonding curve PDA
    pub fn get_signer<'a>(mint: &'a Pubkey, bump: &'a u8) -> [&'a [u8]; 3] {
        [
            Self::SEED_PREFIX.as_bytes(),
            mint.as_ref(),
            std::slice::from_ref(bump),
        ]
    }

    //Update reserves
    pub fn update_reserves(&mut self, reserve_lamport: u64, reserve_token: u64) -> Result<bool> {
        self.virtual_sol_reserve = reserve_lamport;
        self.virtual_token_reserve = reserve_token;

        Ok(true)
    }

    // Swap sol for tokens
    pub fn buy(
        &mut self,
        token_mint: &Account<'info, Mint>,  // Token mint address
        curve_limit: u64,                   // Bonding Curve Limit
        user: &Signer<'info>,               // User address for buyer
        curve_pda: &mut AccountInfo<'info>, // Bonding Curve PDA
        fee_recipient: &mut AccountInfo<'info>, // Team wallet address to get fees
        user_ata: &mut AccountInfo<'info>,  // Associated token account for user
        curve_ata: &AccountInfo<'info>,     // Associated token account for bonding curve
        amount_in: u64,                     // Amount of SOL to pay
        min_amount_out: u64,                // Minimum amount of tokens to receive
        fee_percentage: u64,                // Fee percentage using FEE_PRECISION
        curve_bump: u8,                     // Bump for the bonding curve PDA
        system_program: &AccountInfo<'info>, // System program
        token_program: &AccountInfo<'info>,
    ) -> Result<(u64, u64, u64, u64, u64, bool)> {
        // 1. Calculate amounts and fees
        let (amount_out, fee_amount) =
            self.calculate_amount_out(amount_in, 0, fee_percentage)?;

        // 2. Validate amounts
        require!(
            amount_out >= min_amount_out,
            SwifeyError::InsufficientAmountOut
        );

        // 3. Calculate new reserves
        let new_token_reserves = self
            .virtual_token_reserve
            .checked_sub(amount_out)
            .ok_or(SwifeyError::InvalidReserves)?;

        let new_sol_reserves = self
            .virtual_sol_reserve
            .checked_add(amount_in - fee_amount)
            .ok_or(SwifeyError::InvalidReserves)?;

        // 4. Validate token balance
        let curve_token_balance = curve_ata.try_lamports()?;
        require!(
            curve_token_balance >= amount_out,
            SwifeyError::InsufficientTokenBalance
        );

        // 5. Update reserves
        self.update_reserves(new_sol_reserves, new_token_reserves)?;

        // 6. Perform transfers
        sol_transfer_from_user(&user, fee_recipient, system_program, fee_amount)?;
        sol_transfer_from_user(&user, curve_pda, system_program, amount_in - fee_amount)?;
        token_transfer_with_signer(
            curve_ata,
            curve_pda,
            user_ata,
            token_program,
            &[&BondingCurve::get_signer(&token_mint.key(), &curve_bump)],
            amount_out,
        )?;

        // 7. Check if curve is completed
        let is_completed = if new_sol_reserves >= curve_limit {
            self.is_completed = true;
            true
        } else {
            false
        };

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
    ) -> Result<(u64, u64, u64, u64)> {
        // 1. Calculate amounts and fees
        let (amount_out, fee_amount) =
            self.calculate_amount_out(amount_in, 1, fee_percentage)?;
        
        // 2. Validate amounts
        require!(
            amount_out >= min_amount_out,
            SwifeyError::InsufficientAmountOut
        );

        // 3. Calculate new reserves
        let new_token_reserves = self
            .virtual_token_reserve
            .checked_add(amount_in)
            .ok_or(SwifeyError::InvalidReserves)?;

        let new_sol_reserves = self
            .virtual_sol_reserve
            .checked_sub(amount_out)
            .ok_or(SwifeyError::InvalidReserves)?;

        // 4. Validate SOL balance
        let pda_sol_balance = curve_pda.lamports();
        require!(
            pda_sol_balance >= amount_out,
            SwifeyError::InsufficientSolBalance
        );

        // 5. Update reserves
        self.update_reserves(new_sol_reserves, new_token_reserves)?;

        // 6. Perform transfers
        let token = token_mint.key();
        let signer_seeds: &[&[&[u8]]] = &[&BondingCurve::get_signer(&token, &curve_bump)];

        token_transfer_user(user_ata, curve_ata, user, token_program, amount_in)?;
        sol_transfer_with_signer(
            curve_pda,
            user,      
            system_program,
            signer_seeds,
            amount_out - fee_amount,
        )?;
        sol_transfer_with_signer(
            curve_pda, 
            fee_recipient,
            system_program,
            signer_seeds,
            fee_amount,
        )?;

        Ok((amount_in, amount_out, fee_amount, new_sol_reserves / new_token_reserves))
    }

    //Calculate adjusted amount out and fee amount
    pub fn calculate_amount_out(
        &mut self,
        amount_in: u64,
        direction: u8,
        fee_percentage: u64,    // Now using fixed-point (FEE_PRECISION)
    ) -> Result<(u64, u64)> {
        require!(self.virtual_sol_reserve > 0, SwifeyError::DivisionByZero);
        require!(self.virtual_token_reserve > 0, SwifeyError::DivisionByZero);

        let amount_out = if direction == 0 { // Buying tokens
            // Calculate ratio = (new_sol/current_sol)^CRR
            let current_sol = self.virtual_sol_reserve;
            let new_sol = current_sol.checked_add(amount_in).ok_or(SwifeyError::MathOverflow)?;
            
            let ratio_base = fixed_div(new_sol, current_sol)?;
            let ratio = fixed_pow(ratio_base, CRR_NUMERATOR)?;
            
            // Calculate tokens_out = virtual_token * (1 - 1/ratio)
            let inverse_ratio = fixed_div(PRECISION_U64, ratio)?;
            let one_minus_inverse = PRECISION_U64.checked_sub(inverse_ratio).ok_or(SwifeyError::MathOverflow)?;
            fixed_mul(self.virtual_token_reserve, one_minus_inverse)?
        } else { // Selling tokens
            // For selling, use the inverse formula
            let current_token = self.virtual_token_reserve;
            let new_token = current_token.checked_add(amount_in).ok_or(SwifeyError::MathOverflow)?;
            
            let ratio_base = fixed_div(new_token, current_token)?;
            let inverse_crr = fixed_div(PRECISION_U64, CRR_NUMERATOR)?;
            let ratio = fixed_pow(ratio_base, inverse_crr)?;
            
            let inverse_ratio = fixed_div(PRECISION_U64, ratio)?;
            let one_minus_inverse = PRECISION_U64.checked_sub(inverse_ratio).ok_or(SwifeyError::MathOverflow)?;
            fixed_mul(self.virtual_sol_reserve, one_minus_inverse)?
        };

        // Calculate fee amount using fixed-point arithmetic
        let fee_amount = if direction == 0 {
            fixed_mul(amount_in, fixed_div(fee_percentage, FEE_PRECISION)?)?
        } else {
            fixed_mul(amount_out, fixed_div(fee_percentage, FEE_PRECISION)?)?
        };

        Ok((amount_out, fee_amount))
    }
}
