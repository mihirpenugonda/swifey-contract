use anchor_lang::prelude::*;

#[error_code]
pub enum SwifeyError {
    #[msg("Unauthorized address")]
    UnauthorizedAddress,

    #[msg("Curve limit reached")]
    CurveLimitReached,

    #[msg("Value is not in expected range")]
    IncorrectValueRange,

    #[msg("Amount out is smaller than required amount")]
    InsufficientAmountOut,

    #[msg("An overflow or underflow occurred during calculation")]
    InvalidReserves,

    #[msg("Curve is not initialized")]
    CurveNotInitialized,

    #[msg("Curve is not completed")]
    CurveNotCompleted,
}
