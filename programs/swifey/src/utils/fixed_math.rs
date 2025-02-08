use anchor_lang::prelude::*;
use crate::{errors::SwifeyError, constants::{PRECISION, PRECISION_U64}};

/// Fixed-point multiplication for u128
pub fn fixed_mul_u128(a: u64, b: u128) -> Result<u64> {
    // First multiply, then divide by PRECISION to maintain precision
    let result = (a as u128)
        .checked_mul(b)
        .ok_or_else(|| error!(SwifeyError::MathOverflow))?
        .checked_div(PRECISION)
        .ok_or_else(|| error!(SwifeyError::DivisionByZero))?;

    if result > u64::MAX as u128 {
        return Err(error!(SwifeyError::MathOverflow));
    }

    Ok(result as u64)
}

/// Fixed-point division for u128
pub fn fixed_div_u128(a: u64, b: u64) -> Result<u128> {
    if b == 0 {
        return Err(error!(SwifeyError::DivisionByZero));
    }
    
    let numerator = (a as u128)
        .checked_mul(PRECISION)
        .ok_or_else(|| error!(SwifeyError::MathOverflow))?;

    numerator
        .checked_div(b as u128)
        .ok_or_else(|| error!(SwifeyError::DivisionByZero))
}

/// Legacy fixed-point multiplication for u64
pub fn fixed_mul(a: u64, b: u64) -> Result<u64> {
    // Convert to u128 first to prevent overflow
    let a_u128 = a as u128;
    let b_u128 = b as u128;
    
    // Multiply first
    let full_mul = a_u128
        .checked_mul(b_u128)
        .ok_or_else(|| error!(SwifeyError::MathOverflow))?;
    
    // Then divide by PRECISION
    let result = full_mul
        .checked_div(PRECISION)
        .ok_or_else(|| error!(SwifeyError::DivisionByZero))?;

    if result > u64::MAX as u128 {
        return Err(error!(SwifeyError::MathOverflow));
    }

    Ok(result as u64)
}

/// Legacy fixed-point division for u64
pub fn fixed_div(a: u64, b: u64) -> Result<u64> {
    if b == 0 {
        return Err(error!(SwifeyError::DivisionByZero));
    }
    
    let numerator = (a as u128)
        .checked_mul(PRECISION)
        .ok_or_else(|| error!(SwifeyError::MathOverflow))?;

    let result = numerator
        .checked_div(b as u128)
        .ok_or_else(|| error!(SwifeyError::DivisionByZero))?;

    if result > u64::MAX as u128 {
        return Err(error!(SwifeyError::MathOverflow));
    }

    Ok(result as u64)
}

/// Fixed-point power function using binary exponentiation and scaling
pub fn fixed_pow(base: u64, exp: u64) -> Result<u64> {
    msg!("fixed_pow: Starting with base={}, exp={}", base, exp);
    
    // Validate inputs
    if base == 0 {
        msg!("fixed_pow: Base is zero, returning DivisionByZero error");
        return Err(error!(SwifeyError::DivisionByZero));
    }
    
    // Normalization for large bases instead of saturation
    const MAX_SAFE_BASE: u64 = 10 * PRECISION_U64; // bases above this are considered too high
    if base > MAX_SAFE_BASE {
        msg!("fixed_pow: Base {} exceeds MAX_SAFE_BASE {}. Normalizing.", base, MAX_SAFE_BASE);
        let mut norm_count = 0;
        let mut normalized_base = base;
        while normalized_base > MAX_SAFE_BASE {
            normalized_base = fixed_div(normalized_base, 10)?;
            norm_count += 1;
        }
        msg!("fixed_pow: Normalized base = {} after {} divisions", normalized_base, norm_count);
        let safe_result = fixed_pow(normalized_base, exp)?;
        // Compute correction exponent: (norm_count * (exp as real))
        let exp_multiplier = (norm_count as u64)
            .checked_mul(exp).ok_or_else(|| error!(SwifeyError::MathOverflow))?
            .checked_div(PRECISION_U64).ok_or_else(|| error!(SwifeyError::MathOverflow))?;
        msg!("fixed_pow: Correction exponent = {}", exp_multiplier);
        // In fixed point, 10 is represented as 10 * PRECISION_U64
        let correction = fixed_pow(10 * PRECISION_U64, exp_multiplier)?;
        msg!("fixed_pow: Correction factor = {}", correction);
        let result = fixed_mul(safe_result, correction)?;
        msg!("fixed_pow: Final result after normalization = {}", result);
        return Ok(result);
    }
    
    // For exp = 0, return PRECISION (1.0 in fixed point)
    if exp == 0 {
        msg!("fixed_pow: Exponent is 0, returning PRECISION={}", PRECISION_U64);
        return Ok(PRECISION_U64);
    }
    
    // For exp = PRECISION (1.0), return base unchanged
    if exp == PRECISION_U64 {
        msg!("fixed_pow: Exponent is PRECISION, returning base={}", base);
        return Ok(base);
    }
    
    // For base = PRECISION (1.0), return PRECISION
    if base == PRECISION_U64 {
        msg!("fixed_pow: Base is PRECISION, returning PRECISION={}", PRECISION_U64);
        return Ok(PRECISION_U64);
    }
    
    // Handle fractional exponents
    if exp < PRECISION_U64 {
        msg!("fixed_pow: Handling fractional exponent (exp < PRECISION)");
        
        // First normalize the base to be close to PRECISION
        let base_normalized = if base > PRECISION_U64 {
            msg!("fixed_pow: Normalizing large base {} by dividing with PRECISION", base);
            let normalized = fixed_div(base, PRECISION_U64)?;
            msg!("fixed_pow: Normalized base = {}", normalized);
            normalized
        } else {
            msg!("fixed_pow: Base {} is already normalized", base);
            base
        };

        // Calculate ln(base)
        msg!("fixed_pow: Calculating ln(base_normalized={})", base_normalized);
        let ln_base = fixed_ln(base_normalized)?;
        msg!("fixed_pow: ln(base) = {}", ln_base);
        
        // Calculate exp * ln(base)
        msg!("fixed_pow: Calculating exp({}) * ln_base({})", exp, ln_base);
        let exp_mul = fixed_mul(ln_base, exp)?;
        msg!("fixed_pow: exp * ln(base) = {}", exp_mul);
        
        // Calculate e^(exp * ln(base))
        msg!("fixed_pow: Calculating e^({})", exp_mul);
        let result = fixed_exp(exp_mul)?;
        msg!("fixed_pow: e^(exp * ln(base)) = {}", result);

        // If we normalized the base, we need to adjust the result
        if base > PRECISION_U64 {
            msg!("fixed_pow: Adjusting result for normalized base");
            let scale_factor = fixed_div(base, PRECISION_U64)?;
            msg!("fixed_pow: Scale factor = {}", scale_factor);
            let scale_exp = fixed_mul(exp, PRECISION_U64)?;
            msg!("fixed_pow: Scale exponent = {}", scale_exp);
            let mut final_result = result;
            
            // Apply scaling factor raised to the exponent
            let scale_iterations = scale_exp.checked_div(PRECISION_U64)
                .ok_or_else(|| error!(SwifeyError::DivisionByZero))?;
            msg!("fixed_pow: Applying scale factor {} for {} iterations", scale_factor, scale_iterations);
            
            for i in 0..scale_iterations {
                final_result = fixed_mul(final_result, scale_factor)?;
                msg!("fixed_pow: After iteration {}: final_result = {}", i + 1, final_result);
            }
            msg!("fixed_pow: Final adjusted result = {}", final_result);
            return Ok(final_result);
        }
        msg!("fixed_pow: Returning unadjusted result = {}", result);
        return Ok(result);
    }

    // Handle integer exponent using binary exponentiation
    msg!("fixed_pow: Handling integer exponent");
    let mut result = PRECISION_U64;
    let mut current_exp = exp.checked_div(PRECISION_U64)
        .ok_or_else(|| error!(SwifeyError::DivisionByZero))?;
    let mut current_base = base;

    // Use efficient binary exponentiation (exponentiation by squaring)
    while current_exp > 0 {
        msg!("fixed_pow: Current exp={}, result={}, base={}", current_exp, result, current_base);
        
        if current_exp & 1 == 1 {
            result = fixed_mul(result, current_base)?;
            msg!("fixed_pow: After odd exp multiplication: result={}", result);
        }
        
        if current_exp > 1 {  // Only square if we have more iterations to go
            current_base = fixed_mul(current_base, current_base)?;
            msg!("fixed_pow: After squaring: current_base={}", current_base);
        }
        
        current_exp = current_exp.checked_shr(1)
            .ok_or_else(|| error!(SwifeyError::MathOverflow))?;
    }

    msg!("fixed_pow: Final result = {}", result);
    Ok(result)
}

/// Natural logarithm for fixed-point numbers using Taylor series
pub fn fixed_ln(x: u64) -> Result<u64> {
    if x == 0 {
        return Err(error!(SwifeyError::DivisionByZero));
    }
    
    // If x is very close to PRECISION (1.0), return 0
    if x == PRECISION_U64 {
        return Ok(0);
    }

    // For x < 1, use ln(x) = -ln(1/x)
    if x < PRECISION_U64 {
        let inverse = fixed_div(PRECISION_U64, x)?;
        let ln_inverse = fixed_ln(inverse)?;
        return Ok(PRECISION_U64.checked_sub(ln_inverse)
            .ok_or_else(|| error!(SwifeyError::MathOverflow))?);
    }

    // Normalize x to be close to 1 for better convergence
    let mut shift = 0u64;
    let mut normalized_x = x;
    while normalized_x >= PRECISION_U64.checked_mul(2)
        .ok_or_else(|| error!(SwifeyError::MathOverflow))? {
        normalized_x = fixed_div(normalized_x, 2 * PRECISION_U64)?;
        shift = shift.checked_add(1)
            .ok_or_else(|| error!(SwifeyError::MathOverflow))?;
    }

    // Use Taylor series for ln(1 + y) where y = x - 1
    let y = normalized_x.checked_sub(PRECISION_U64)
        .ok_or_else(|| error!(SwifeyError::MathOverflow))?;
    let mut sum = 0u64;
    let mut term = y;
    let mut n = 1u64;
    
    while term > 0 && n < 10 {
        if n % 2 == 1 {
            sum = sum.checked_add(fixed_div(term, n * PRECISION_U64)?)
                .ok_or_else(|| error!(SwifeyError::MathOverflow))?;
        } else {
            sum = sum.checked_sub(fixed_div(term, n * PRECISION_U64)?)
                .ok_or_else(|| error!(SwifeyError::MathOverflow))?;
        }
        term = fixed_mul(term, y)?;
        n = n.checked_add(1)
            .ok_or_else(|| error!(SwifeyError::MathOverflow))?;
    }

    // Add back the shift component: ln(x) = ln(x/2^k) + k*ln(2)
    if shift > 0 {
        let ln2 = 693147180559945309; // ln(2) * PRECISION
        let shift_component = shift.checked_mul(ln2)
            .ok_or_else(|| error!(SwifeyError::MathOverflow))?;
        sum = sum.checked_add(shift_component)
            .ok_or_else(|| error!(SwifeyError::MathOverflow))?;
    }
    
    Ok(sum)
}

/// Exponential function for fixed-point numbers using Taylor series
pub fn fixed_exp(x: u64) -> Result<u64> {
    // If x is 0, return PRECISION (1.0)
    if x == 0 {
        return Ok(PRECISION_U64);
    }

    // Limit input range to prevent overflow
    if x > 41446531673968666 { // ln(2^64) * PRECISION
        return Err(error!(SwifeyError::MathOverflow));
    }

    let mut sum = PRECISION_U64;
    let mut term = PRECISION_U64;
    let mut n = 1u64;
    
    while term > 0 && n < 15 {
        term = fixed_mul(term, fixed_div(x, n * PRECISION_U64)?)?;
        sum = sum.checked_add(term)
            .ok_or_else(|| error!(SwifeyError::MathOverflow))?;
        n = n.checked_add(1)
            .ok_or_else(|| error!(SwifeyError::MathOverflow))?;
    }
    
    Ok(sum)
}

/// Calculate fee amount using fixed-point arithmetic
pub fn calculate_fee_amount(amount: u64, fee_percentage: u64) -> Result<u64> {
    fixed_mul(amount, fee_percentage)
}

// Unit tests for fixed_pow
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_pow_integer_exponent() {
         // For base = 2.0 (represented as 2 * PRECISION_U64) and exponent = 1.0 (PRECISION_U64),
         // the result should be roughly 2.0 in fixed point.
         let base = 2 * PRECISION_U64;
         let exp = PRECISION_U64; // 1.0
         let result = fixed_pow(base, exp).unwrap();
         // Allow a small rounding error
         assert!((result as i64 - (2 * PRECISION_U64) as i64).abs() < 10);
    }

    #[test]
    fn test_fixed_pow_fractional_exponent() {
         // For base = 4.0 and exponent = 0.5, the result should be roughly 2.0
         let base = 4 * PRECISION_U64;
         let exp = PRECISION_U64 / 2; // 0.5
         let result = fixed_pow(base, exp).unwrap();
         assert!((result as i64 - (2 * PRECISION_U64) as i64).abs() < 10);
    }

    #[test]
    fn test_fixed_pow_normalization() {
         // Test a large base value that requires normalization.
         let base = 1000 * PRECISION_U64; // large base
         let exp = PRECISION_U64; // exponent 1.0
         let result = fixed_pow(base, exp).unwrap();
         // Expected result should be approximately base. Allow for some rounding error.
         assert!((result as i64 - (1000 * PRECISION_U64) as i64).abs() < 1000);
    }
} 