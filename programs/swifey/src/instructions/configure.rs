use crate::{errors::SwifeyError, states::{Config, ConfigSettings}};
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
        
        // Log the current state
        msg!("Current authority: {}", self.global_config.authority);
        msg!("Signer: {}", self.admin.key());

        // If config is not initialized, set the admin as the authority
        if !self.global_config.authority.eq(&Pubkey::default()) {
            require!(
                self.global_config.authority == self.admin.key(),
                SwifeyError::UnauthorizedAddress
            );
        } 

        // Copy all fields from ConfigSettings to Config
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
        self.global_config.is_paused = new_config.is_paused;
        self.global_config.reserved = new_config.reserved;
        Ok(())
    }
}