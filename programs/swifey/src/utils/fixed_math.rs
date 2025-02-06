use anchor_lang::prelude::*;
use crate::{errors::SwifeyError, constants::{PRECISION, PRECISION_U64}};

/// Fixed-point multiplication for u128
pub fn fixed_mul_u128(a: u64, b: u128) -> Result<u64> {
    let result = (a as u128)
        .checked_mul(b)
        .ok_or(SwifeyError::MathOverflow)?
        .checked_div(PRECISION)
        .ok_or(SwifeyError::MathOverflow)?;

    if result > u64::MAX as u128 {
        return Err(SwifeyError::MathOverflow.into());
    }

    Ok(result as u64)
}

/// Fixed-point division for u128
pub fn fixed_div_u128(a: u64, b: u64) -> Result<u128> {
    let numerator = (a as u128)
        .checked_mul(PRECISION)
        .ok_or(SwifeyError::MathOverflow)?;

    numerator
        .checked_div(b as u128)
        .ok_or(SwifeyError::MathOverflow.into())
}

/// Legacy fixed-point multiplication for u64 (for backwards compatibility)
pub fn fixed_mul(a: u64, b: u64) -> Result<u64> {
    let result = (a as u128)
        .checked_mul(b as u128)
        .ok_or(SwifeyError::MathOverflow)?
        .checked_div(PRECISION)
        .ok_or(SwifeyError::MathOverflow)?;

    if result > u64::MAX as u128 {
        return Err(SwifeyError::MathOverflow.into());
    }

    Ok(result as u64)
}

/// Legacy fixed-point division for u64 (for backwards compatibility)
pub fn fixed_div(a: u64, b: u64) -> Result<u64> {
    let numerator = (a as u128)
        .checked_mul(PRECISION)
        .ok_or(SwifeyError::MathOverflow)?;

    let result = numerator
        .checked_div(b as u128)
        .ok_or(SwifeyError::MathOverflow)?;

    if result > u64::MAX as u128 {
        return Err(SwifeyError::MathOverflow.into());
    }

    Ok(result as u64)
}

/// Simple power function for fixed-point numbers
pub fn fixed_pow(base: u64, exp: u64) -> Result<u64> {
    // Convert to regular numbers first (remove fixed-point scaling)
    let base_float = (base as f64) / (PRECISION as f64);
    let exp_float = (exp as f64) / (PRECISION as f64);
    
    // Calculate power
    let result = base_float.powf(exp_float);
    
    // Convert back to fixed-point
    let result_fixed = (result * PRECISION as f64) as u64;
    
    Ok(result_fixed)
}

/// Natural logarithm for fixed-point numbers
pub fn fixed_ln(x: u64) -> Result<u64> {
    require!(x > 0, SwifeyError::DivisionByZero);
    
    // Use Taylor series for ln(1 + x)
    let mut sum = 0u64;
    let mut term = x;
    let mut n = 1u64;
    
    while term > 0 && n < 10 {
        if n % 2 == 1 {
            sum = sum.checked_add(fixed_div(term, n * PRECISION_U64)?).ok_or(SwifeyError::MathOverflow)?;
        } else {
            sum = sum.checked_sub(fixed_div(term, n * PRECISION_U64)?).ok_or(SwifeyError::MathOverflow)?;
        }
        term = fixed_mul(term, x)?;
        n = n.checked_add(1).ok_or(SwifeyError::MathOverflow)?;
    }
    
    Ok(sum)
}

/// Exponential function for fixed-point numbers
pub fn fixed_exp(x: u64) -> Result<u64> {
    // Use Taylor series for e^x
    let mut sum = PRECISION_U64;
    let mut term = PRECISION_U64;
    let mut n = 1u64;
    
    while term > 0 && n < 10 {
        term = fixed_mul(term, fixed_div(x, n * PRECISION_U64)?)?;
        sum = sum.checked_add(term).ok_or(SwifeyError::MathOverflow)?;
        n = n.checked_add(1).ok_or(SwifeyError::MathOverflow)?;
    }
    
    Ok(sum)
}

/// Calculate fee amount using fixed-point arithmetic
pub fn calculate_fee_amount(amount: u64, fee_percentage: u64) -> Result<u64> {
    fixed_mul(amount, fee_percentage)
} 