use crate::{errors::SwifeyError, states::Config};
use anchor_lang::{prelude::*, system_program};

#[derive(Accounts)]
pub struct Configure<'info> {
    #[account(mut)]
    admin: Signer<'info>,

    #[account(
        init_if_needed,
        payer = admin,
        seeds = [Config::SEED_PREFIX.as_bytes()],
        space = 8 + Config::LEN,
        bump
    )]
    global_config: Account<'info, Config>,

    #[account(address = system_program::ID)]
    system_program: Program<'info, System>,
}

impl<'info> Configure<'info> {
    pub fn process(&mut self, new_config: Config) -> Result<()> {
        // Verify authority if already initialized    
        if !self.global_config.authority.eq(&Pubkey::default()) {
            require!(
                self.global_config.authority.eq(&self.admin.key()),
                SwifeyError::UnauthorizedAddress
            );
        }

        // Verify that 80% of total supply is allocated to bonding curve
        let required_curve_supply = (new_config.total_token_supply as f64 * 0.8) as u64;
        require!(
            new_config.initial_virtual_token_reserve >= required_curve_supply,
            SwifeyError::InvalidTokenAllocation
        );

        // Verify curve parameters for 42 SOL target
        const TARGET_SOL: u64 = 42_000_000_000; // 42 SOL in lamports
        const INITIAL_SOL: u64 = 10_500_000_000; // 10.5 SOL in lamports

        require!(
            new_config.curve_limit == TARGET_SOL,
            SwifeyError::InvalidCurveLimit
        );

        require!(
            new_config.initial_virtual_sol_reserve == INITIAL_SOL,
            SwifeyError::InvalidInitialSolReserve
        );

        self.global_config.set_inner(new_config);
        Ok(())
    }
}
