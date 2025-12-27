# Concentrated Liquidity Market Maker (CLMM) - Complete Mathematical Guide

## Table of Contents
1. [Introduction](#introduction)
2. [Core Concepts](#core-concepts)
3. [Mathematical Formulas](#mathematical-formulas)
4. [How Everything is Related](#how-everything-is-related)
5. [Complete Example](#complete-example)
6. [How Liquidity Providers Make Money](#how-liquidity-providers-make-money)

---

## Introduction

A **Concentrated Liquidity Market Maker (CLMM)** is an advanced automated market maker (AMM) that allows liquidity providers (LPs) to concentrate their capital within specific price ranges, rather than spreading it across all possible prices (0 to ∞).

**Why is this better?**
- More capital efficiency (you can use less money to provide the same depth)
- Higher fee earnings for LPs
- Better prices for traders
- Popularized by Uniswap V3

---

## Core Concepts

### 1. Fixed Point Arithmetic (Q64)

Computers can't handle decimals perfectly, so we use **fixed-point math**:

```
Q64 = 2^64 = 18,446,744,073,709,551,616
```

**Example:**
- Price = 1.5
- Stored as: 1.5 × 2^64 = 27,670,116,110,564,327,424

This gives us precision up to 18 decimal places!

### 2. Square Root Prices

Prices are stored as **square roots** for mathematical convenience:

```
sqrt_price_x64 = √(price) × Q64
```

**Why square roots?**
- The constant product formula becomes simpler
- Calculations are more numerically stable
- Reduces computational errors

**Example:**
- Price: 100 USDC per SOL
- sqrt_price_x64 = √100 × 2^64 = 10 × 2^64 = 184,467,440,737,095,516,160

### 3. Ticks

A **tick** represents a discrete price level:

```
price = 1.0001^tick
```

**Properties:**
- Each tick is 0.01% apart (1 basis point)
- Tick 0 → price = 1.0
- Tick 100 → price = 1.0001^100 ≈ 1.0101 (1.01% higher)
- Tick -100 → price = 1.0001^-100 ≈ 0.9900 (0.99% lower)
- MIN_TICK = -443,636 (extremely low price)
- MAX_TICK = 443,636 (extremely high price)

**Tick Spacing = 10** means liquidity positions must align with ticks that are multiples of 10 (-100, -90, -80, etc.)

---

## Mathematical Formulas

### Formula 1: Constant Product Formula (Foundation)

The AMM maintains this invariant:

```
x × y = k (constant)
```

Where:
- `x` = amount of token A in pool
- `y` = amount of token B in pool
- `k` = constant (liquidity squared)

**With square root prices:**

```
L = √(x × y)
```

Where `L` is liquidity (the geometric mean of reserves).

---

### Formula 2: Price to Square Root Price

Converting regular price to sqrt_price_x64:

```
sqrt_price_x64 = √(price) × 2^64
```

**Implementation:**
1. Calculate `√price` using Newton's method
2. Multiply by Q64 (2^64)

**Example:**
```
price = 100
√100 = 10
sqrt_price_x64 = 10 × 2^64 = 184,467,440,737,095,516,160
```

---

### Formula 3: Tick to Square Root Price

Converting tick to price:

```
price = 1.0001^tick
sqrt_price = √(1.0001^tick) = 1.0001^(tick/2)
```

**Key optimization:** Use binary decomposition!

Instead of calculating 1.0001^443636, we decompose:
- 443,636 in binary = 1101100010101010100₂
- Calculate 1.0001^(2^0), 1.0001^(2^1), 1.0001^(2^2), ..., 1.0001^(2^18)
- Multiply only the powers where bit = 1

**Example:**
```
tick = 5 = 101₂
sqrt_price = 1.0001^(2.5)
           = 1.0001^(1) × 1.0001^(4)  (skip 2^1 because bit is 0)
```

This reduces 443,636 multiplications to just 19!

---

### Formula 4: Square Root Price to Tick (Inverse)

Converting price back to tick:

```
tick = log₁.₀₀₀₁(price)
     = log₂(price) / log₂(1.0001)
```

**Implementation steps:**
1. Calculate log₂(sqrt_price_x64) using bit-by-bit approximation
2. Find integer part (MSB position)
3. Find fractional part (iterative refinement)
4. Convert from log₂ to log₁.₀₀₀₁ using change of base
5. Verify with forward calculation

---

### Formula 5: Liquidity to Token Amounts

Given liquidity `L` and price range `[P_lower, P_upper]`, calculate required tokens:

#### Case 1: Current Price Below Range (P_current ≤ P_lower)
**All Token A:**

```
amount_a = L × (√P_upper - √P_lower) × Q64 / (√P_upper × √P_lower)
amount_b = 0
```

#### Case 2: Current Price Above Range (P_current ≥ P_upper)
**All Token B:**

```
amount_a = 0
amount_b = L × (√P_upper - √P_lower) / Q64
```

#### Case 3: Current Price Within Range (P_lower < P_current < P_upper)
**Both Tokens:**

```
amount_a = L × (√P_upper - √P_current) × Q64 / (√P_upper × √P_current)
amount_b = L × (√P_current - √P_lower) / Q64
```

---

### Formula 6: Swap Calculations

#### A → B Swap (Selling A, Price Decreases)

**Price moves from P_current down to P_next:**

```
Δx = L × Δ(1/√P) × Q64
   = L × (√P_current - √P_next) × Q64 / (√P_current × √P_next)

Δy = L × Δ√P / Q64
   = L × (√P_current - √P_next) / Q64
```

Where:
- `Δx` = Token A input (you give)
- `Δy` = Token B output (you receive)

**To find next price given input amount:**

```
√P_next = (L × √P_current) / (L + Δx × √P_current / Q64)
```

#### B → A Swap (Buying A, Price Increases)

**Price moves from P_current up to P_next:**

```
Δy = L × Δ√P / Q64
   = L × (√P_next - √P_current) / Q64

Δx = L × Δ(1/√P) × Q64
   = L × (√P_next - √P_current) × Q64 / (√P_current × √P_next)
```

Where:
- `Δy` = Token B input (you give)
- `Δx` = Token A output (you receive)

**To find next price given input amount:**

```
√P_next = √P_current + (Δy × Q64 / L)
```

---

## How Everything is Related

### The Flow of Calculations:

```
                    PRICE
                      |
                      |
        +-------------+-------------+
        |                           |
        v                           v
    TICK (discrete)         SQRT_PRICE_X64 (continuous)
        |                           |
        |                           |
        v                           v
  LIQUIDITY RANGE              SWAP CALCULATIONS
        |                           |
        |                           |
        v                           v
  TOKEN AMOUNTS                PRICE IMPACT
```

### Step-by-Step Relationship:

1. **Start with Price:**
   - Traders see: "1 SOL = 100 USDC"
   
2. **Convert to Tick:**
   - `tick = log₁.₀₀₀₁(100) ≈ 46,054`
   
3. **Round to Tick Spacing:**
   - With spacing = 10: tick = 46,050
   
4. **Convert Tick to Sqrt Price:**
   - `sqrt_price_x64 = 1.0001^(46050/2) × 2^64`
   
5. **LP Provides Liquidity:**
   - Range: [90, 110] USDC per SOL
   - Liquidity: 1,000,000 units
   
6. **Calculate Token Amounts:**
   - Use Formula 5 with current price = 100
   - Determines how much SOL and USDC needed
   
7. **Trader Swaps:**
   - Sells 10 SOL for USDC
   - Use Formula 6 to calculate:
     - New price
     - USDC received
     - Fee collected
   
8. **Price Changes:**
   - New price affects all active liquidity positions
   - Positions may move in/out of range

---

## Complete Example

### Setup: SOL/USDC Pool

**Initial State:**
- Current Price: 100 USDC per SOL
- Tick: 46,054
- Liquidity in range [90, 110]: L = 1,000,000

### Step 1: Alice Provides Liquidity

**Alice's Position:**
- Price Range: [95, 105] USDC per SOL
- Liquidity to Provide: L = 100,000

**Calculate Ticks:**
```
tick_lower = log₁.₀₀₀₁(95) ≈ 45,950 → round to 45,950
tick_upper = log₁.₀₀₀₁(105) ≈ 46,150 → round to 46,150
```

**Convert to Sqrt Prices:**
```
sqrt_price_lower = 1.0001^(45950/2) × 2^64 ≈ 179,831,471,...
sqrt_price_current = 1.0001^(46054/2) × 2^64 ≈ 184,467,440,...
sqrt_price_upper = 1.0001^(46150/2) × 2^64 ≈ 189,177,123,...
```

**Calculate Required Tokens (Formula 5 - Case 3):**

Since 95 < 100 < 105, Alice needs both tokens:

```
amount_SOL = L × (√P_upper - √P_current) × Q64 / (√P_upper × √P_current)
           = 100,000 × (√105 - √100) × 2^64 / (√105 × √100)
           ≈ 100,000 × 0.2469 × 2^64 / 102.47
           ≈ 4.45 SOL

amount_USDC = L × (√P_current - √P_lower) / Q64
            = 100,000 × (√100 - √95) / 2^64
            ≈ 100,000 × 0.2512 / 2^64
            ≈ 448.6 USDC
```

**Alice deposits: ~4.45 SOL + ~448.6 USDC**

### Step 2: Bob Swaps (Sells SOL for USDC)

**Bob's Trade:**
- Sells: 2 SOL
- Direction: A → B (price goes down)
- Fee: 0.3%

**Calculate Swap (Formula 6):**

Target price is lower bound of current liquidity range (or user's minimum acceptable price).

Let's say target = 99 USDC per SOL

```
sqrt_price_target = √99 × 2^64 ≈ 183,522,287,...
```

**Check if we can reach target with available liquidity:**

```
required_SOL = L × (√P_current - √P_target) × Q64 / (√P_current × √P_target)
             = 1,100,000 × (√100 - √99) × 2^64 / (√100 × √99)
             ≈ 1,100,000 × 0.0503 × 2^64 / 994.99
             ≈ 1.11 SOL
```

Bob has 2 SOL, which is more than 1.11 SOL needed!

**Full Step (reach target):**
```
next_price = 99 USDC per SOL
amount_in = 1.11 SOL
```

**Calculate USDC Output:**
```
amount_out = L × (√P_current - √P_next) / Q64
           = 1,100,000 × (√100 - √99) / 2^64
           ≈ 1,100,000 × 0.0503 / 2^64
           ≈ 110.11 USDC
```

**After Fee (0.3%):**
```
Fee = 110.11 × 0.003 = 0.33 USDC
Bob receives = 110.11 - 0.33 = 109.78 USDC
```

**Second Swap Step for Remaining 0.89 SOL:**
(Process repeats for remaining amount until all SOL is swapped)

### Step 3: Pool State After Swap

**Before Swap:**
- SOL in pool: ~1,000 SOL
- USDC in pool: ~100,000 USDC
- Price: 100 USDC/SOL

**After Swap:**
- SOL in pool: ~1,001.11 SOL (received 1.11 from Bob)
- USDC in pool: ~99,889.89 USDC (sent 110.11 to Bob)
- Price: 99 USDC/SOL
- Fees collected: 0.33 USDC

### Step 4: Alice's Position After Swap

Alice's range is [95, 105], and current price is 99 (still in range).

**Her position value changed:**

```
New amount_SOL = 100,000 × (√105 - √99) × Q64 / (√105 × √99)
               ≈ 4.58 SOL (increased!)

New amount_USDC = 100,000 × (√99 - √95) / Q64
                ≈ 428.4 USDC (decreased!)
```

**Alice also earned fees:**
- Her share = (100,000 / 1,100,000) × 0.33 USDC = 0.03 USDC

### Step 5: Alice Withdraws Liquidity

When Alice withdraws her 100,000 liquidity:

**She receives:**
- 4.58 SOL
- 428.4 USDC
- 0.03 USDC (accumulated fees)

**Compare to initial deposit:**
- Started: 4.45 SOL + 448.6 USDC
- Ended: 4.58 SOL + 428.43 USDC

**What happened?**
- More SOL (4.58 vs 4.45) → +0.13 SOL
- Less USDC (428.43 vs 448.6) → -20.17 USDC
- Plus fees: +0.03 USDC

**Net change:**
- +0.13 SOL ≈ +12.87 USDC (at price 99)
- -20.17 USDC
- +0.03 USDC fees
- **Total: -7.27 USDC (impermanent loss)**

But she earned fees! With more volume, fees > impermanent loss.

---

## How Liquidity Providers Make Money

### Revenue Streams

#### 1. Trading Fees (Main Income)

**How it works:**
- Every swap pays a fee (typically 0.01% to 1%)
- Fees are distributed proportionally to LPs based on their liquidity share
- Fees accumulate automatically

**Example:**
```
Your liquidity: 100,000 units
Total liquidity in range: 1,000,000 units
Your share: 10%

Trade happens: 10,000 USDC volume, 0.3% fee = 30 USDC fee
Your earnings: 10% × 30 = 3 USDC
```

**In concentrated liquidity:**
- You only earn when price is in YOUR range
- But you earn MORE per unit of capital (higher capital efficiency)

#### 2. Arbitrage Rebalancing

When price changes, your position rebalances automatically:
- Price goes up → you sell the appreciating token
- Price goes down → you buy the depreciating token

This is "selling high and buying low" automatically!

### Costs and Risks

#### 1. Impermanent Loss (IL)

**What is it?**
When price moves, your position value can be less than if you just held the tokens.

**Formula:**
```
IL = 2 × √(price_ratio) / (1 + price_ratio) - 1
```

**Example:**
- Start: 1 SOL + 100 USDC (total = 200 USDC)
- Price doubles: 1 SOL now worth 200 USDC
- If you held: 1 SOL + 100 USDC = 300 USDC
- In pool: ~0.707 SOL + 141.42 USDC = 282.84 USDC
- IL = (282.84 - 300) / 300 = -5.7%

**But in concentrated liquidity:**
- IL is amplified when price leaves your range
- But you earn more fees to compensate

#### 2. Out of Range Risk

If price moves outside your range:
- You stop earning fees
- Your position becomes 100% one token
- You need to rebalance manually

### Profit Calculation Example

**Alice's complete trade:**

**Initial Investment (at price 100):**
- 4.45 SOL × 100 = 445 USDC
- 448.6 USDC
- **Total: 893.6 USDC**

**After 1 week (price now 99):**
- 4.58 SOL × 99 = 453.42 USDC
- 428.4 USDC
- Fees: 15 USDC (from week of trading)
- **Total: 896.82 USDC**

**Profit Analysis:**
```
Final Value: 896.82 USDC
Initial Value: 893.6 USDC
Profit: 3.22 USDC

Breakdown:
- Impermanent Loss: -7.27 USDC
- Trading Fees: +15 USDC
- Net Profit: +7.73 USDC (0.87% return in 1 week!)
```

**Annualized Return:** ~45% APY (if fee rate continues)

### Optimal Strategies for LPs

#### 1. Choose the Right Range

**Narrow Range (e.g., [98, 102]):**
- ✅ Higher fee earnings (10x more concentrated)
- ❌ Higher IL risk
- ❌ Goes out of range more easily
- **Best for:** Stablecoins, low volatility pairs

**Wide Range (e.g., [50, 200]):**
- ✅ Lower IL risk
- ✅ Stays in range longer
- ❌ Lower fee earnings
- **Best for:** Volatile pairs, passive strategies

#### 2. Active Management

**Rebalancing:**
- When price moves, adjust your range
- Stay where the trading volume is
- Compound your fees

**Example:**
```
Week 1: Price 100, range [95, 105] → 15 USDC fees
Week 2: Price 110, range [105, 115] → 18 USDC fees (rebalanced)
Week 3: Price 105, range [100, 110] → 20 USDC fees (rebalanced)
```

#### 3. Fee Tier Selection

Different pools have different fee tiers:
- **0.01%:** Stablecoin pairs (high volume, low volatility)
- **0.05%:** Correlated pairs (ETH/WBTC)
- **0.3%:** Standard pairs (most common)
- **1%:** Exotic pairs (high volatility)

Higher fees compensate for higher IL risk.

### Real-World Example: Successful LP

**Pool:** USDC/USDT (stablecoin pair)
**Range:** [0.998, 1.002] (very tight!)
**Liquidity:** $100,000
**Fee Tier:** 0.01%

**Daily Stats:**
- Volume: $50,000,000
- Your share: 0.1% ($50,000 of liquidity)
- Fees collected: $50M × 0.01% × 0.1% = $500/day

**Annual Calculation:**
```
Daily fees: $500
Annual fees: $500 × 365 = $182,500
Investment: $100,000
APY: 182.5%
```

This is possible because:
1. Stablecoins stay in tight range (low IL)
2. High trading volume (lots of fees)
3. Concentrated liquidity (high capital efficiency)

---

## Summary

### Key Formulas to Remember:

1. **Price = 1.0001^tick**
2. **sqrt_price_x64 = √price × 2^64**
3. **Liquidity = √(x × y)**
4. **Amount_A = L × Δ(1/√P) × Q64**
5. **Amount_B = L × Δ√P / Q64**

### When LPs Make Money:

✅ High trading volume in their range
✅ Fees > Impermanent Loss
✅ Active range management
✅ Appropriate risk tolerance for volatility

### When LPs Lose Money:

❌ Price moves far outside range
❌ Low trading volume
❌ High impermanent loss, low fees
❌ Gas costs exceed earnings (on expensive chains)

**The Golden Rule:** Fees must outpace impermanent loss. Choose your ranges and pairs wisely!