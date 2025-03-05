use anchor_lang::prelude::*;
use crate::errors::SwifeyError;
use crate::states::*;

#[derive(Accounts)]
pub struct GetUserEscrows<'info> {
    /// The user whose escrows we want to retrieve
    pub user: Signer<'info>,
    
    #[account(
        seeds = [
            ESCROW_REGISTRY_SEED,
            user.key().as_ref(),
        ],
        bump,
    )]
    pub user_registry: Account<'info, EscrowRegistry>,
}

impl<'info> GetUserEscrows<'info> {
    pub fn process(&self) -> Result<()> {
        // This is a view function, so we don't need to modify any state
        // The client can read the user_registry account data to get the escrows
        Ok(())
    }
}

pub fn get_user_escrows(ctx: Context<GetUserEscrows>) -> Result<()> {
    ctx.accounts.process()
} 