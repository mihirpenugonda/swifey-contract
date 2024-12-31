use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::utils::{
    sol_transfer_from_user, sol_transfer_with_signer, token_transfer_user,
    token_transfer_with_signer,
};
use crate::errors::SwifeyError;

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
}

impl<'info> BondingCurve {
    pub const SEED_PREFIX: &'static str = "bonding_curve";
    pub const LEN: usize = 8 * 5 + 1 + 1;

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
        token_mint: &Account<'info, Mint>, // Token mint address
        curve_limit: u64, // Bonding Curve Limit
        user: &Signer<'info>, // User address for buyer
        curve_pda: &mut AccountInfo<'info>, // Bonding Curve PDA
        fee_recipient: &mut AccountInfo<'info>, // Team wallet address to get fees  
        user_ata: &mut AccountInfo<'info>, // Associated token account for user
        curve_ata: &AccountInfo<'info>, // Associated token account for bonding curve
        amount_in: u64, // Amount of SOL to pay
        min_amount_out: u64, // Minimum amount of tokens to receive
        fee_percentage: f64, // Fee percentage for buying on the bonding curve
        curve_bump: u8, // Bump for the bonding curve PDA
        system_program: &AccountInfo<'info>, // System program
        token_program: &AccountInfo<'info>,
    ) -> Result<bool> {
        let (amount_out, fee_amount) =
            self.calculate_amount_out(amount_in, token_mint.decimals, 0, fee_percentage)?;

        // Check if the amount out is greater than the minimum amount out
        require!(amount_out >= min_amount_out, SwifeyError::InsufficientAmountOut);

        // Transfer fee to the team wallet
        sol_transfer_from_user(&user, fee_recipient, system_program, fee_amount)?;
        
        // Transfer adjusted amount to curve 
        sol_transfer_from_user(&user, curve_pda, system_program, amount_in - fee_amount)?;

        // Transfer tokens from PDA to user
        token_transfer_with_signer(
            curve_ata,
            curve_pda,
            user_ata,
            token_program,
            &[&BondingCurve::get_signer(&token_mint.key(), &curve_bump)],
            amount_out,
        )?;

        // Calculate new reserves
        let new_token_reserves = self
            .virtual_token_reserve
            .checked_sub(amount_out)
            .ok_or(SwifeyError::InvalidReserves)?;

        let new_sol_reserves = self
            .virtual_sol_reserve
            .checked_add(amount_in - fee_amount)
            .ok_or(SwifeyError::InvalidReserves)?;

        msg!(
            "New reserves:: Token: {:?}, Sol: {:?}",
            new_token_reserves,
            new_sol_reserves
        );

        //Update reserves on the curve
        self.update_reserves(new_sol_reserves, new_token_reserves)?;
        
        //Return true if curve reached its limit
        if new_sol_reserves >= curve_limit {
            self.is_completed = true;
            return Ok(true);
        }

        //Return false if curve did not reach its limit
        Ok(false)
    }

    // Swap tokens for sol
    pub fn sell(
        &mut self,
        token_mint: &Account<'info, Mint>, // Token mint address
        user: &Signer<'info>, // User address for seller
        curve_pda: &mut AccountInfo<'info>, // Bonding Curve PDA
        user_ata: &mut AccountInfo<'info>, // Associated token account for user
        fee_recipient: &mut AccountInfo<'info>, // Team wallet address to get fees  
        curve_ata: &mut AccountInfo<'info>, // Associated token account for bonding curve
        amount_in: u64, // SOL Amount to pay
        min_amount_out: u64, // Minimum amount out 
        fee_percentage: f64, // Selling fee percentage
        curve_bump: u8, // Bump for the bonding curve PDA
        system_program: &AccountInfo<'info>, // System program
        token_program: &AccountInfo<'info>, // Token program
    ) -> Result<()> {
        let (amount_out, fee_amount) =
            self.calculate_amount_out(amount_in, token_mint.decimals, 1, fee_percentage)?;

        // Check if the amount out is greater than the minimum amount out
        require!(amount_out >= min_amount_out, SwifeyError::InsufficientAmountOut);

        let token = token_mint.key();

        let signer_seeds: &[&[&[u8]]] = &[&BondingCurve::get_signer(&token, &curve_bump)];

        // Transfer fee to the team wallet
        sol_transfer_with_signer(
            &user,
            fee_recipient,
            system_program,
            signer_seeds,
            fee_amount,
        )?;

        // Transfer adjusted amount to curve 
        sol_transfer_with_signer(
            &user,
            curve_pda,
            system_program,
            signer_seeds,
            amount_in - fee_amount,
        )?;
        
        // Transfer token from user to PDA
        token_transfer_user(user_ata, user, curve_ata, token_program, amount_out)?;

        // Calculate new reserves
        let new_token_reserves = self
            .virtual_token_reserve
            .checked_add(amount_in)
            .ok_or(SwifeyError::InvalidReserves)?;

        let new_sol_reserves = self
            .virtual_sol_reserve
            .checked_sub(amount_out + fee_amount)
            .ok_or(SwifeyError::InvalidReserves)?;

        msg!(
            "New reserves:: Token: {:?}, Sol: {:?}",
            new_token_reserves,
            new_sol_reserves
        );

        // Update reserves on the curve
        self.update_reserves(new_sol_reserves, new_token_reserves)?;

        Ok(())
    }

    //Calculate adjusted amount out and fee amount
    pub fn calculate_amount_out(
        &mut self,
        amount_in: u64,
        token_decimal: u8,
        direction: u8,
        fee_percentage: f64,
    ) -> Result<(u64, u64)> {
        let fee_amount = (amount_in as f64 * fee_percentage / 100.0) as u64;
        let amount_after_fee = amount_in
            .checked_sub(fee_amount)
            .ok_or(SwifeyError::InsufficientFunds)?;

        // Convert to u128 for larger number handling
        let virtual_sol = self.virtual_sol_reserve as u128;
        let virtual_token = self.virtual_token_reserve as u128;
        let amount_after_fee = amount_after_fee as u128;

        // Calculate dynamic reserve ratio based on current reserves
        let reserve_ratio = if virtual_sol > 0 {
            virtual_token
                .checked_mul(1_000_000)  // Scale for precision (PPM)
                .and_then(|r| r.checked_div(virtual_sol))
                .ok_or(SwifeyError::InvalidReserves)?
        } else {
            return Err(SwifeyError::InvalidReserves.into());
        };

        println!("Dynamic reserve ratio: {}", reserve_ratio);

        let amount_out = if direction == 0 {
            // Buy tokens
            let tokens_out = amount_after_fee
                .checked_mul(virtual_token)
                .and_then(|r| r.checked_mul(reserve_ratio))
                .and_then(|r| r.checked_div(1_000_000)) // Remove PPM scaling
                .and_then(|r| r.checked_div(virtual_sol))
                .ok_or(SwifeyError::InvalidReserves)?;

            tokens_out
        } else {
            // Sell tokens
            let sol_out = amount_after_fee
                .checked_mul(virtual_sol)
                .and_then(|r| r.checked_mul(reserve_ratio))
                .and_then(|r| r.checked_div(1_000_000)) // Remove PPM scaling
                .and_then(|r| r.checked_div(virtual_token))
                .ok_or(SwifeyError::InvalidReserves)?;

            sol_out
        };

        let final_amount = u64::try_from(amount_out).map_err(|_| SwifeyError::IncorrectValueRange)?;
        println!("Calculated amount out: {}", final_amount);

        let adjusted_amount = if direction == 0 {
            final_amount
                .checked_mul(10u64.pow(token_decimal as u32))
                .ok_or(SwifeyError::InvalidReserves)?
        } else {
            final_amount
        };

        Ok((adjusted_amount, fee_amount))
    }
}
