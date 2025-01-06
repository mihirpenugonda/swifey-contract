use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod states;
pub mod utils;

use crate::instructions::*;
use crate::states::*;

declare_id!("ANuqfyR8ETpfr1EhZ7ZbqDo1p53qoP6X6bykC168Gaa4");

#[program]
pub mod swifey {
    use super::*;

    pub fn configure(ctx: Context<Configure>, new_config: Config) -> Result<()> {
        ctx.accounts.process(new_config)
    }

    pub fn launch<'info>(ctx: Context<'_, '_, '_, 'info, Launch<'info>>,
        name: String,
        symbol: String,
        uri: String
    ) -> Result<()> {
        ctx.accounts.process(name, symbol, uri, ctx.bumps.global_config)
    }

    pub fn swap<'info>(ctx: Context<'_, '_, '_, 'info, Swap<'info>>, amount: u64, direction: u8, min_out: u64) -> Result<()> {
        ctx.accounts.process(amount, direction, min_out, ctx.bumps.bonding_curve)
    }

    pub fn migrate(
        ctx: Context<Migrate>,
        params: MigrateParams
    ) -> Result<()> {
        Migrate::process(ctx, params)
    }

}