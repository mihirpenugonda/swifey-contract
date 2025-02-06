use crate::{errors::SwifeyError, states::{Config, ConfigSettings}, utils::{ConfigurationUpdated, ConfigurationInitialized}, constants::{FEE_PRECISION, LAMPORTS_PER_SOL}};
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
    pub fn process(&mut self, new_config: ConfigSettings) -> Result<()> {
        msg!("Processing configuration update...");
        
        // Validate configuration parameters
        // 1. Validate fee percentages are within reasonable bounds (0-100%)
        require!(
            new_config.buy_fee_percentage <= FEE_PRECISION && 
            new_config.sell_fee_percentage <= FEE_PRECISION && 
            new_config.migration_fee_percentage <= FEE_PRECISION,
            SwifeyError::InvalidFeePercentage
        );

        // 2. Validate initial virtual token reserve is at least 80% of total supply
        require!(
            new_config.initial_virtual_token_reserve >= (new_config.total_token_supply * 80) / 100,
            SwifeyError::InvalidTokenAllocation
        );

        // 3. Validate initial SOL reserve is non-zero and reasonable
        require!(
            new_config.initial_virtual_sol_reserve >= LAMPORTS_PER_SOL, // At least 1 SOL
            SwifeyError::InsufficientLiquidity
        );

        // 4. Validate curve limit is greater than initial SOL reserve
        require!(
            new_config.curve_limit > new_config.initial_virtual_sol_reserve,
            SwifeyError::InvalidCurveLimit
        );

        // 5. Validate total token supply is non-zero
        require!(
            new_config.total_token_supply > 0,
            SwifeyError::InvalidTokenAllocation
        );

        // 6. Validate initial real token reserve is zero or less than virtual reserve
        require!(
            new_config.initial_real_token_reserve <= new_config.initial_virtual_token_reserve,
            SwifeyError::InvalidTokenAllocation
        );

        // Check if this is first-time initialization
        let is_initialization = self.global_config.authority.eq(&Pubkey::default());
        
        if !is_initialization {
            // If not initialization, verify authority
            require!(
                self.global_config.authority == self.admin.key(),
                SwifeyError::UnauthorizedAddress
            );

            // Store old values for event emission
            let old_authority = self.global_config.authority;
            let old_fee_recipient = self.global_config.fee_recipient;
            let old_curve_limit = self.global_config.curve_limit;
            let old_buy_fee_percentage = self.global_config.buy_fee_percentage;
            let old_sell_fee_percentage = self.global_config.sell_fee_percentage;
            let old_migration_fee_percentage = self.global_config.migration_fee_percentage;
            let old_is_paused = self.global_config.is_paused;

            // Update configuration
            self.update_config(&new_config);

            // Emit update event
            emit!(ConfigurationUpdated {
                admin: self.admin.key(),
                old_authority,
                new_authority: new_config.authority,
                old_fee_recipient,
                new_fee_recipient: new_config.fee_recipient,
                old_curve_limit,
                new_curve_limit: new_config.curve_limit,
                old_buy_fee_percentage,
                new_buy_fee_percentage: new_config.buy_fee_percentage,
                old_sell_fee_percentage,
                new_sell_fee_percentage: new_config.sell_fee_percentage,
                old_migration_fee_percentage,
                new_migration_fee_percentage: new_config.migration_fee_percentage,
                old_is_paused,
                new_is_paused: new_config.is_paused,
                timestamp: Clock::get()?.unix_timestamp,
            });
        } else {
            // First time initialization
            msg!("Initializing configuration for the first time");
            
            // Update configuration
            self.update_config(&new_config);

            // Emit initialization event
            emit!(ConfigurationInitialized {
                admin: self.admin.key(),
                authority: new_config.authority,
                fee_recipient: new_config.fee_recipient,
                curve_limit: new_config.curve_limit,
                initial_virtual_token_reserve: new_config.initial_virtual_token_reserve,
                initial_virtual_sol_reserve: new_config.initial_virtual_sol_reserve,
                initial_real_token_reserve: new_config.initial_real_token_reserve,
                total_token_supply: new_config.total_token_supply,
                buy_fee_percentage: new_config.buy_fee_percentage,
                sell_fee_percentage: new_config.sell_fee_percentage,
                migration_fee_percentage: new_config.migration_fee_percentage,
                is_paused: new_config.is_paused,
                timestamp: Clock::get()?.unix_timestamp,
            });
        }

        Ok(())
    }

    // Helper function to update configuration
    fn update_config(&mut self, new_config: &ConfigSettings) {
        self.global_config.authority = new_config.authority;
        self.global_config.fee_recipient = new_config.fee_recipient;
        self.global_config.curve_limit = new_config.curve_limit;
        self.global_config.initial_virtual_token_reserve = new_config.initial_virtual_token_reserve;
        self.global_config.initial_virtual_sol_reserve = new_config.initial_virtual_sol_reserve;
        self.global_config.initial_real_token_reserve = new_config.initial_real_token_reserve;
        self.global_config.total_token_supply = new_config.total_token_supply;
        self.global_config.buy_fee_percentage = new_config.buy_fee_percentage;
        self.global_config.sell_fee_percentage = new_config.sell_fee_percentage;
        self.global_config.migration_fee_percentage = new_config.migration_fee_percentage;
        self.global_config.max_price_impact = new_config.max_price_impact;
        self.global_config.is_paused = new_config.is_paused;
        self.global_config.reserved = new_config.reserved;
    }
}