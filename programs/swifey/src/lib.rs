use anchor_lang::prelude::*;

declare_id!("tUkgCLXftGcKozmxWY6UuiED9hMsow9HuimVUPjNtZS");

#[program]
pub mod swifey {
    use super::*;

    pub fn launch_token<'info>(ctx: Context<'_, '_, '_, 'info, LaunchToken<'info>>,
        name: String,
        symbol: String,
        uri: String,
        mint_authority: Pubkey,
    ) -> Result<()> {
        ctx.accounts.process(name, symbol, uri, mint_authority)
    }
}

#[derive(Accounts)]
pub struct Initialize {
    
}
