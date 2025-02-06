use anchor_lang::prelude::*;

#[event]
pub struct MigrationCompleted {
    pub token_mint: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub migration_fee: u64,
    pub raydium_pool: Pubkey,
}

#[event]
pub struct TokenSold {
    pub token_mint: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub fee_amount: u64,
    pub price: u64
}

#[event]
pub struct TokenPurchased {
    pub token_mint: Pubkey,
    pub buyer: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub fee_amount: u64,
    pub price: u64
}

#[event]
pub struct CurveCompleted {
    pub token_mint: Pubkey,
    pub final_sol_reserve: u64,
    pub final_token_reserve: u64,
}

#[event]
pub struct ConfigurationInitialized {
    pub admin: Pubkey,
    pub authority: Pubkey,
    pub fee_recipient: Pubkey,
    pub curve_limit: u64,
    pub initial_virtual_token_reserve: u64,
    pub initial_virtual_sol_reserve: u64,
    pub initial_real_token_reserve: u64,
    pub total_token_supply: u64,
    pub buy_fee_percentage: u64,
    pub sell_fee_percentage: u64,
    pub migration_fee_percentage: u64,
    pub is_paused: bool,
    pub timestamp: i64,
}

#[event]
pub struct ConfigurationUpdated {
    pub admin: Pubkey,
    pub old_authority: Pubkey,
    pub new_authority: Pubkey,
    pub old_fee_recipient: Pubkey,
    pub new_fee_recipient: Pubkey,
    pub old_curve_limit: u64,
    pub new_curve_limit: u64,
    pub old_buy_fee_percentage: u64,
    pub new_buy_fee_percentage: u64,
    pub old_sell_fee_percentage: u64,
    pub new_sell_fee_percentage: u64,
    pub old_migration_fee_percentage: u64,
    pub new_migration_fee_percentage: u64,
    pub old_is_paused: bool,
    pub new_is_paused: bool,
    pub timestamp: i64,
}
