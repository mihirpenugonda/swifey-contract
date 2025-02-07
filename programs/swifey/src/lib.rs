use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod states;
pub mod utils;

use crate::instructions::*;
use crate::states::*;

declare_id!("JBrV73n2b37ivcQJiNEPsKrjSSFKq1hk1vSFPMMYR9wx");

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
}