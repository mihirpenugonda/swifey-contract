use anchor_lang::prelude::*;

declare_id!("tUkgCLXftGcKozmxWY6UuiED9hMsow9HuimVUPjNtZS");

#[program]
pub mod swifey {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
