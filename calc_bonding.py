def calculate_bonding_curve(initial_sol_amount, target_sol_amount, crr, total_tokens):
    """
    Calculate bonding curve parameters and token distribution
    
    Args:
        initial_sol_amount (float): Initial SOL amount in SOL (not lamports)
        target_sol_amount (float): Target SOL amount in SOL (not lamports)
        crr (float): Constant Reserve Ratio (between 0 and 1)
        total_tokens (int): Total tokens allocated to bonding curve
    """
    # Convert SOL to lamports
    initial_sol = initial_sol_amount * 1e9
    target_sol = target_sol_amount * 1e9
    SOL_PRICE = 210  # Current SOL price in USD
    
    # Calculate price ratio
    ratio = (target_sol/initial_sol)**crr
    
    # Calculate tokens that would be sold
    tokens_sold = total_tokens * (1 - 1/ratio)
    
    # Calculate initial and final prices
    initial_price = initial_sol/(total_tokens * crr)
    final_price = target_sol/(total_tokens * crr)
    
    print("\nBonding Curve Analysis:")
    print("=" * 50)
    print(f"Parameters:")
    print(f"Initial SOL: {initial_sol_amount} SOL (${initial_sol_amount * SOL_PRICE:,.2f})")
    print(f"Target SOL: {target_sol_amount} SOL (${target_sol_amount * SOL_PRICE:,.2f})")
    print(f"CRR: {crr}")
    print(f"Total Tokens Allocated: {total_tokens:,}")
    
    print("\nPrice Analysis:")
    print("-" * 50)
    print(f"Initial price: {initial_price/1e9:.8f} SOL per token (${initial_price/1e9*SOL_PRICE:.6f})")
    print(f"Final price: {final_price/1e9:.8f} SOL per token (${final_price/1e9*SOL_PRICE:.6f})")
    print(f"Price increase ratio: {ratio:.4f}x")
    
    print("\nToken Distribution Results:")
    print("-" * 50)
    print(f"Tokens that would be sold: {tokens_sold:,.0f}")
    print(f"Percentage of allocation sold: {(tokens_sold/total_tokens)*100:.2f}%")
    
    print("\nDetailed Price and Distribution Analysis:")
    print("-" * 110)
    print("SOL Amount | Tokens Sold | % of Total | Price (SOL) | Price ($) | % Price Incr. | Market Cap ($)")
    print("-" * 110)
    
    # Calculate distribution at key SOL levels with 10% more granularity
    key_percentages = [i for i in range(0, 101, 10)]  # 0%, 10%, 20%, ..., 100%
    sol_levels = [initial_sol_amount + (target_sol_amount - initial_sol_amount) * (p/100) for p in key_percentages]
    
    initial_price_lamports = initial_price
    
    for sol in sol_levels:
        ratio = (sol*1e9/initial_sol)**crr
        tokens = total_tokens * (1 - 1/ratio)
        price = (sol*1e9)/(total_tokens * crr)
        price_increase = ((price - initial_price_lamports) / initial_price_lamports * 100) if initial_price_lamports > 0 else 0
        market_cap = (price/1e9 * SOL_PRICE) * total_tokens  # Total market cap including unsold tokens
        
        print(f"{sol:9.1f} | {tokens:10,.0f} | {(tokens/total_tokens)*100:8.1f}% | {price/1e9:.8f} | ${price/1e9*SOL_PRICE:.6f} | {price_increase:8.1f}% | ${market_cap:,.2f}")

if __name__ == "__main__":
    # Example usage with current parameters
    TOTAL_TOKENS = 1_000_000_000
    INITIAL_SOL = 12.5
    TARGET_SOL = 100
    CRR = 0.651
    
    print("\nTest Case 1: Current Parameters")
    calculate_bonding_curve(INITIAL_SOL, TARGET_SOL, CRR, TOTAL_TOKENS)
    
    # Uncomment to try different parameters
    # print("\nTrying different parameters:")
    # calculate_bonding_curve(5.0, 42, 0.9, TOTAL_TOKENS) 