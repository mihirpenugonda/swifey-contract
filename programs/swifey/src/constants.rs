pub const TOKEN_DECIMAL: u8 = 6; //Token decimal

// Fixed-point math constants
pub const PRECISION: u128 = 1_000_000_000_000; // 12 decimal places for fixed-point
pub const PRECISION_U64: u64 = 1_000_000_000_000;
pub const CRR_NUMERATOR: u64 = 605_100_000_000; // 0.6051 * PRECISION
pub const FEE_PRECISION: u64 = 10_000; // 100.00% = 10000

// Bonding Curve Parameters
pub const TARGET_SOL_AMOUNT: u64 = 42_000_000_000; // 42 SOL in lamports
pub const INITIAL_SOL_RESERVE: u64 = 12_330_000_000; // 12.33 SOL in lamports
pub const TOKEN_RESERVE_PERCENTAGE: u64 = 8_000; // 80% = 8000 with FEE_PRECISION
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000; // Conversion rate for lamports to SOL
