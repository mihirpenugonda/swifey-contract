use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ConfigSettings {  // New struct for the instruction argument
    pub authority: Pubkey,
    pub fee_recipient: Pubkey,
    pub curve_limit: u64,
    pub initial_virtual_token_reserve: u64,
    pub initial_virtual_sol_reserve: u64,
    pub initial_real_token_reserve: u64,
    pub total_token_supply: u64,
    pub buy_fee_percentage: u64,     // Uses FEE_PRECISION (10000 = 100.00%)
    pub sell_fee_percentage: u64,    // Uses FEE_PRECISION (10000 = 100.00%)
    pub migration_fee_percentage: u64, // Uses FEE_PRECISION (10000 = 100.00%)
    pub is_paused: bool,             // New pause flag
    pub reserved: [[u8; 8]; 8]
}

#[account]
pub struct Config {
    pub authority: Pubkey, // Primary authority address
    pub fee_recipient: Pubkey, // Team wallet address
    pub curve_limit: u64, // Lamports to complete the bonding curve

    // Curve token/sol amount reserves
    pub initial_virtual_token_reserve: u64,
    pub initial_virtual_sol_reserve: u64,
    pub initial_real_token_reserve: u64,
    pub total_token_supply: u64,

    // Fee percentages using FEE_PRECISION (10000 = 100.00%)
    pub buy_fee_percentage: u64,
    pub sell_fee_percentage: u64,
    pub migration_fee_percentage: u64,

    pub is_paused: bool,             // New pause flag
    pub reserved: [[u8; 8]; 8]
}

impl Config {
    pub const SEED_PREFIX: &'static str = "global_config";
    pub const LEN: usize = 32 + // authority
        32 + // fee_recipient
        8 + // curve_limit
        8 + // initial_virtual_token_reserve
        8 + // initial_virtual_sol_reserve
        8 + // initial_real_token_reserve
        8 + // total_token_supply
        8 + // buy_fee_percentage
        8 + // sell_fee_percentage
        8 + // migration_fee_percentage
        1 + // is_paused
        64; // reserved
}