use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod states;
pub mod utils;

use crate::instructions::*;
use crate::states::*;

declare_id!("79UdtbaGWUkLePZGMmkG6RDAxXKExEhkxDLmmjBxxq1g");

#[program]
pub mod swifey {
    use super::*;

    pub fn configure(ctx: Context<Configure>, new_config: ConfigSettings) -> Result<()> {
        ctx.accounts.process(new_config)
    }

    pub fn launch<'info>(ctx: Context<'_, '_, '_, 'info, Launch<'info>>,
        name: String,
        symbol: String,
        uri: String
    ) -> Result<()> {
        instructions::launch(ctx, name, symbol, uri)
    }

    pub fn swap<'info>(
        ctx: Context<'_, '_, '_, 'info, Swap<'info>>, 
        amount: u64, 
        direction: u8, 
        min_out: u64
    ) -> Result<()> {
        instructions::swap(ctx, amount, direction, min_out)
    }

    pub fn migrate(ctx: Context<Migrate>) -> Result<()> {
        Migrate::process(ctx)
    }
    
    pub fn create_escrow(
        ctx: Context<CreateEscrow>,
        amount: u64,
        expires_at: Option<i64>,
    ) -> Result<()> {
        instructions::create_escrow(ctx, amount, expires_at)
    }
    
    pub fn close_escrow(
        ctx: Context<CloseEscrow>,
        distribution_percentages: Vec<Distribution>,
    ) -> Result<()> {
        instructions::close_escrow(ctx, distribution_percentages)
    }
    
    pub fn close_escrow_multi<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, CloseEscrowMulti<'info>>,
        distribution_percentages: Vec<Distribution>,
    ) -> Result<()> {
        instructions::close_escrow_multi(ctx, distribution_percentages)
    }
    
    pub fn get_user_escrows(ctx: Context<GetUserEscrows>) -> Result<()> {
        instructions::get_user_escrows(ctx)
    }
}