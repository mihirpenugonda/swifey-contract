pub const TOKEN_DECIMAL: u8 = 6; //Token decimal

// Fixed-point math constants
pub const PRECISION: u128 = 1_000_000_000_000;  // 10^12
pub const PRECISION_U64: u64 = 1_000_000_000;   // Reduced to 10^9 for u64 operations
pub const FEE_PRECISION: u64 = 10_000;  // 100% = 10000

// CRR (Constant Reserve Ratio) = 0.651
pub const CRR_NUMERATOR: u64 = 6510;
pub const CRR_DENOMINATOR: u64 = 10000;

// Bonding Curve Parameters
pub const TARGET_SOL_AMOUNT: u64 = 100_000_000_000; // 100 SOL in lamports
pub const INITIAL_SOL_RESERVE: u64 = 12_500_000_000; // 12.5 SOL in lamports
pub const TOKEN_RESERVE_PERCENTAGE: u64 = 10_000; // 100% = 10000 with FEE_PRECISION
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000; // Conversion rate for lamports to SOL

// Minimum amounts
pub const MIN_BUY_AMOUNT: u64 = 1_000_000;  // 0.001 SOL
pub const MIN_SELL_AMOUNT: u64 = 1_000;      // 0.001 tokens

// Price impact limits
pub const MAX_PRICE_IMPACT_BPS: u64 = 1000;  // 10%
