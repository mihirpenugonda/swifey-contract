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
        msg!("global_config.authority: {:?}", self.global_config.authority);
        msg!("admin.key(): {:?}", self.admin.key());
        if !self.global_config.authority.eq(&Pubkey::default()) {
            require!(
                self.global_config.authority.eq(&self.admin.key()),
                SwifeyError::UnauthorizedAddress
            );
        }

        self.global_config.set_inner(new_config);
        Ok(())
    }
}
