use anchor_lang::prelude::*;
use crate::errors::SwifeyError;
use crate::constants::{PRECISION, PRECISION_U64};

/// Fixed-point multiplication
pub fn fixed_mul(a: u64, b: u64) -> Result<u64> {
    let result = (a as u128)
        .checked_mul(b as u128)
        .ok_or(SwifeyError::MathOverflow)?
        .checked_div(PRECISION)
        .ok_or(SwifeyError::MathOverflow)?;
    
    Ok(result as u64)
}

/// Fixed-point division
pub fn fixed_div(a: u64, b: u64) -> Result<u64> {
    require!(b != 0, SwifeyError::DivisionByZero);
    
    let result = (a as u128)
        .checked_mul(PRECISION)
        .ok_or(SwifeyError::MathOverflow)?
        .checked_div(b as u128)
        .ok_or(SwifeyError::MathOverflow)?;
    
    Ok(result as u64)
}

/// Calculate power with fixed-point base and exponent
pub fn fixed_pow(base: u64, exp: u64) -> Result<u64> {
    if exp == 0 {
        return Ok(PRECISION_U64);
    }
    if exp == PRECISION_U64 {
        return Ok(base);
    }
    
    // For fractional exponents, we use the following formula:
    // a^(n/d) = exp(ln(a) * n/d)
    let ln_base = fixed_ln(base)?;
    let exp_scaled = fixed_mul(ln_base, exp)?;
    fixed_exp(exp_scaled)
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