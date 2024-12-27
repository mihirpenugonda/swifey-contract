use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

#[account]
pub struct BondingCurve {
    pub virtual_token_reserve: u64,
    pub virtual_sol_reserve: u64,
    pub real_token_reserve: u64,
    pub real_sol_reserve: u64,
    pub token_total_supply: u64,
    pub is_completed: bool
}

impl<'info> BondingCurve<'info> {
    pub const SEED_PREFIX: &'static str = "bonding-curve";
    pub const LEN: usize = 8 * 5 + 1;

    pub fn get_signer<'a>(mint: &'a Pubkey, bump: &'a u8) -> [&'a [u8]; 3] {
        [
            Self::SEED_PREFIX.as_bytes(),
            mint.as_ref(),
            std::slice::from_ref(bump)
        ]
    }

    pub fn update_reserves(&mut self, reserve_lamport: u64, reserve_token: u64) -> Result<bool> {
        self.virtual_sol_reserve = reserve_lamport;
        self.virtual_token_reserve = reserve_token;

        Ok(true)
    }

    pub fn buy(&mut self, token_mint: &Account<'info, Mint>, curve_limit: u64, user: &Signer<'info>, curve_pda: &mut AccountInfo<'info>, fee_recipient: &mut AccountInfo<'info>, user_ata: &mut AccountInfo<'info>, curve_ata: &AccountInfo<'info>, amount_in: u64, min_amount_out: u64, fee_percentage: f64, curve_bump: u8, system_program: &AccountInfo<'info>, token_program: &AccountInfo<'info>) -> Result<bool> {
        
    }

    pub fn calculate_amount_out(&mut self, amount_in: u64, token_decimal: u8, direction: u8, fee_percentage: f64) ->Result<(u64, u64)> {
        let fee_amount = (amount_in as f64 * fee_percentage / 100.0) as u64;
        let amount_after_fee = amount_in.checked_sub(fee_amount).ok_or(Error::InsufficientFunds)?;

        let amount_in_u129 = amount_after_fee as u128;
        let virtual_sol = self.virtual_sol_reserve as u128;
        let virtual_token = self.virtual_token_reserve as u128;

        const PRECISION: u128 = 1_000_000;
        const WEIGHT: u128 = 500_000;

        let amount_out = if direction == 0 {
            if virtual_sol == 0 || virtual_token == 0 {
                return Err(Error::InvalidReserves.into());
            }

            let sol_ratio = (virtual_sol * PRECISION).checked_div(virtual_sol.checked_add(amount_in_u128).ok_or(Error::InvalidReserves)?).ok_or(Error::InvalidReserves)?;

            let power = (sol_ration as f64 / PRECISION as f64).powf(WEIGHT as f64 / PRECISION as f64);
            let tokens_out = ((1.0 - power) * virtual_token as f64) as u128;


            if tokens_out > virtual_token {
                return Err(Error::InvalidReserves.into());
            }

            tokens_out
        } else {
            if virtual_sol == 0 || virtual_token == 0 {
                return Err(Error::InvalidReserves.into());
            }

            let token_ratio = (virtual_token * PRECISION).checked_div(virtual_token.checked_add(amount_in_u128).ok_or(Error::InvalidReserves)?).ok_or(Error::InvalidReserves)?;

            let power = (token_ration as f64 / PRECISION as f64).powf(PRECISION as f64 / WEIGHT as f64);
            let sol_out = ((1.0 - power) * virtual_sol as f64) as u128;

            if sol_out > virtual_sol {
                return Err(Error::InvalidReserves.into());
            }
            sol_out
        };

        let final_amount = u64::try_from(amount_out).map_err(|_| Error::InvalidAmountOut)?;

        require!(final_amount > 0, Error::InvalidAmountOut);

        let adjusted_amount = if direction == 0 {
            final_amount.checked_mul(10u64.pow(token_decimal as u32)).ok_or(Error::InvalidAmountOut)?
        } else{
            final_amount
        };

        OK((adjusted_amount, fee_lamports))
    }
}
