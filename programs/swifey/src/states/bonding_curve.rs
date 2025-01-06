use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::errors::SwifeyError;
use crate::utils::{
    sol_transfer_from_user, sol_transfer_with_signer, token_transfer_user,
    token_transfer_with_signer,
};

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
        token_mint: &Account<'info, Mint>,  // Token mint address
        curve_limit: u64,                   // Bonding Curve Limit
        user: &Signer<'info>,               // User address for buyer
        curve_pda: &mut AccountInfo<'info>, // Bonding Curve PDA
        fee_recipient: &mut AccountInfo<'info>, // Team wallet address to get fees
        user_ata: &mut AccountInfo<'info>,  // Associated token account for user
        curve_ata: &AccountInfo<'info>,     // Associated token account for bonding curve
        amount_in: u64,                     // Amount of SOL to pay
        min_amount_out: u64,                // Minimum amount of tokens to receive
        fee_percentage: f64,                // Fee percentage for buying on the bonding curve
        curve_bump: u8,                     // Bump for the bonding curve PDA
        system_program: &AccountInfo<'info>, // System program
        token_program: &AccountInfo<'info>,
    ) -> Result<bool> {
        let (amount_out, fee_amount) =
            self.calculate_amount_out(amount_in, 0, fee_percentage)?;

        // Check if the amount out is greater than the minimum amount out
        require!(
            amount_out >= min_amount_out,
            SwifeyError::InsufficientAmountOut
        );

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
        token_mint: &Account<'info, Mint>,
        user: &Signer<'info>,
        curve_pda: &mut AccountInfo<'info>,
        user_ata: &mut AccountInfo<'info>,
        fee_recipient: &mut AccountInfo<'info>,
        curve_ata: &mut AccountInfo<'info>,
        amount_in: u64,
        min_amount_out: u64,
        fee_percentage: f64,
        curve_bump: u8,
        system_program: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
    ) -> Result<()> {
        msg!("=== SELL OPERATION START ===");
        msg!("User: {}", user.key());
        msg!("Fee Recipient: {}", fee_recipient.key());
        msg!("Curve PDA: {}", curve_pda.key());
        msg!("User ATA: {}", user_ata.key());
        msg!("Curve ATA: {}", curve_ata.key());
        msg!("Amount In: {}", amount_in);
        msg!("Min Amount Out: {}", min_amount_out);
        msg!("Fee Percentage: {}", fee_percentage);
        msg!("Token Program: {}", token_program.key());
        let (amount_out, fee_amount) =
            self.calculate_amount_out(amount_in, 1, fee_percentage)?;
        
        msg!("Calculated amount_out: {}", amount_out);
        msg!("Calculated fee_amount: {}", fee_amount);

        require!(
            amount_out >= min_amount_out,
            SwifeyError::InsufficientAmountOut
        );

        let token = token_mint.key();
        let signer_seeds: &[&[&[u8]]] = &[&BondingCurve::get_signer(&token, &curve_bump)];
        msg!("Generated signer seeds for curve PDA");

        msg!("Attempting token transfer FROM user TO curve...");
        msg!("User ATA balance before transfer: {}", user_ata.lamports());
        msg!("Curve ATA balance before transfer: {}", curve_ata.lamports());
        token_transfer_user(user_ata, curve_ata, user, token_program, amount_in)?;
        msg!("Token transfer successful");

        msg!("Attempting SOL transfer FROM curve TO user...");
        msg!("Curve PDA balance before transfer: {}", curve_pda.lamports());
        msg!("User balance before transfer: {}", user.lamports());
        sol_transfer_with_signer(
            curve_pda,
            user,      
            system_program,
            signer_seeds,
            amount_out - fee_amount,
        )?;
        msg!("SOL transfer to user successful");

        msg!("Attempting fee transfer FROM curve TO fee recipient...");
        msg!("Fee recipient balance before transfer: {}", fee_recipient.lamports());
        sol_transfer_with_signer(
            curve_pda, 
            fee_recipient,
            system_program,
            signer_seeds,
            fee_amount,
        )?;
        msg!("Fee transfer successful");

        let new_token_reserves = self
            .virtual_token_reserve
            .checked_add(amount_in) 
            .ok_or(SwifeyError::InvalidReserves)?;

        let new_sol_reserves = self
            .virtual_sol_reserve
            .checked_sub(amount_out)
            .ok_or(SwifeyError::InvalidReserves)?;

        msg!(
            "New reserves calculation successful:: Token: {}, Sol: {}",
            new_token_reserves,
            new_sol_reserves
        );

        msg!("Updating reserves on the curve...");
        self.update_reserves(new_sol_reserves, new_token_reserves)?;
        msg!("Reserves updated successfully");

        msg!("=== SELL OPERATION COMPLETE ===");
        Ok(())
    }

    //Calculate adjusted amount out and fee amount
    pub fn calculate_amount_out(
        &mut self,
        amount_in: u64,
        direction: u8,
        fee_percentage: f64,    
    ) -> Result<(u64, u64)> {
        let fee_amount = (amount_in as f64 * fee_percentage / 100.0) as u64;
        let amount_after_fee = amount_in
            .checked_sub(fee_amount)
            .ok_or(SwifeyError::InsufficientFunds)?;

        let virtual_sol = self.virtual_sol_reserve as f64;
        let virtual_token = self.virtual_token_reserve as f64;
        let amount_after_fee = amount_after_fee as f64;

        const CRR: f64 = 0.2;

        let amount_out = if direction == 0 {
            let base = 1.0 + amount_after_fee / virtual_sol;
            virtual_token * (base.powf(CRR) - 1.0)
        } else {
            let base = 1.0 - amount_after_fee / virtual_token;
            virtual_sol * (1.0 - base.powf(1.0 / CRR))
        };

        let final_amount = amount_out.floor() as u64;

        msg!("Final amount before decimal adjustment: {}", final_amount);

        Ok((final_amount, fee_amount))
    }
}
