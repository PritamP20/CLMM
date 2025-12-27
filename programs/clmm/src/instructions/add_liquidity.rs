use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer, mint_to};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::Pool;
use crate::state::Tick;
use crate::error::CLMMError;
use crate::utils::{TICK_SPACING, calculate_liquidity_amounts, integer_sqrt, tick_to_sqrt_price_x64};


#[derive(Accounts)]
#[instruction(tick_lower:i32,tick_upper:i32)]
pub struct AddLiquidity<'info>{
    #[account(mut)]
    pub signer:Signer<'info>,

    /// CHECK: This is a PDA used as authority
    #[account(seeds=[b"authority", mint_a.key().as_ref(), mint_b.key().as_ref()], bump)]
    pub authority:UncheckedAccount<'info>,

    #[account(
        mut,
        seeds=[b"pool", mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump
    )]
    pub pool:AccountLoader<'info, Pool>,

    #[account(
        mut,
        seeds=[b"tick", mint_a.key().as_ref(), mint_b.key().as_ref(), &tick_lower.to_le_bytes()],
        bump
    )]
    pub tick_lower_account:Account<'info, Tick>,
    #[account(
        mut,
        seeds=[b"tick", mint_a.key().as_ref(), mint_b.key().as_ref(), &tick_upper.to_le_bytes()],
        bump
    )]
    pub tick_upper_account:Account<'info, Tick>,
    
    pub mint_a:Account<'info, Mint>,
    pub mint_b:Account<'info, Mint>,

    #[account(
        mut,
        seeds=[b"vault_token", mint_a.key().as_ref(), mint_b.key().as_ref(),b"A"],
        bump,
        token::mint=mint_a,
        token::authority=authority
    )]
    pub vault_a:Account<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint=mint_a,
        associated_token::authority=signer,
    )]
    pub token_account_a:Account<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint=mint_b,
        associated_token::authority=signer
    )]
    pub token_account_b:Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds=[b"vault_token", mint_a.key().as_ref(), mint_b.key().as_ref(), b"B"],
        bump,
        token::mint=mint_b,
        token::authority=authority
    )]
    pub vault_b:Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds=[b"lp_token", mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump
    )]
    pub lp_token_mint:Account<'info, Mint>,

    #[account(
        init,
        payer=signer,
        associated_token::mint=lp_token_mint,
        associated_token::authority=signer
    )]
    pub lp_token_account:Account<'info, TokenAccount>,
    
    pub associated_token_program:Program<'info,AssociatedToken>,
    pub token_program:Program<'info, Token>,
    pub system_program:Program<'info, System>
}

pub fn add_liquidity(
    ctx:Context<AddLiquidity>,
    tick_lower:i32,
    tick_upper:i32,
    liquidity:u128,
)->Result<()>{
    require!(tick_lower < tick_upper, CLMMError::TickMismatch);
    
    let mut pool=ctx.accounts.pool.load_mut()?;
    let tick_lower_acc=&mut ctx.accounts.tick_lower_account;
    let tick_upper_acc=&mut ctx.accounts.tick_upper_account;
    let token_a_mint = ctx.accounts.mint_a.key();
    let token_b_mint = ctx.accounts.mint_b.key();

    require_eq!(tick_lower_acc.index, tick_lower, CLMMError::InvalidTickIndex);
    require_eq!(tick_upper_acc.index, tick_upper, CLMMError::InvalidTickIndex);

    require!(pool.mint_a==token_a_mint, CLMMError::InvalidTokenMint);
    require!(pool.mint_b==token_b_mint, CLMMError::InvalidTokenMint);

    require!(
        tick_lower%TICK_SPACING as i32==0 && tick_upper%TICK_SPACING as i32==0,
        CLMMError::UnalignedTick
    );

    tick_lower_acc.liquidity_net=tick_lower_acc.liquidity_net.checked_add(liquidity as i128).ok_or(CLMMError::ArithmeticOverflow)?;
    tick_upper_acc.liquidity_net=tick_upper_acc.liquidity_net.checked_sub(liquidity as i128).ok_or(CLMMError::ArithmeticOverflow)?;

    if pool.current_tick >=tick_lower && pool.current_tick < tick_upper{
        pool.active_liquidity=pool.active_liquidity.checked_add(liquidity).ok_or(CLMMError::ArithmeticOverflow)?; // can raise a pr here
    }
    let authority_seeds = &[
        b"authority",
        token_a_mint.as_ref(),
        token_b_mint.as_ref(),
        &[ctx.bumps.authority],
    ];
    let sqrt_price_lower_x64 = tick_to_sqrt_price_x64(tick_lower)?;
    let sqrt_price_upper_x64 = tick_to_sqrt_price_x64(tick_upper)?;
    let sqrt_price_current_x64 = pool.sqrt_price_x64;

    let (amount_a, amount_b) = calculate_liquidity_amounts(sqrt_price_current_x64, sqrt_price_lower_x64, sqrt_price_upper_x64, liquidity)?;

    if amount_a != 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.token_account_a.to_account_info(),
                    to: ctx.accounts.vault_a.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            ),
            amount_a,
        )?;
    }

    if amount_b != 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.token_account_b.to_account_info(),
                    to: ctx.accounts.vault_b.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            ),
            amount_b,
        )?;
    }

    let mint_amount = if pool.total_lp_issued == 0 {
        if amount_a > 0 && amount_b > 0 {
            let product = (amount_a as u128).checked_mul(amount_b as u128).ok_or(CLMMError::ArithmeticOverflow)?;
            integer_sqrt(product) as u128
        } else {
            std::cmp::max(amount_a, amount_b) as u128
        }
    } else {
        let pool_balance_a = ctx.accounts.vault_a.amount;
        let pool_balance_b = ctx.accounts.vault_b.amount;
        if pool_balance_a == 0 && pool_balance_b == 0 {
            return Err(CLMMError::PoolEmpty.into());
        }
        let share_from_a = if pool_balance_a > 0 {
            (amount_a as u128)
                .checked_mul(pool.total_lp_issued as u128)
                .ok_or(CLMMError::ArithmeticOverflow)?
                .checked_div(pool_balance_a as u128)
                .ok_or(CLMMError::ArithmeticOverflow)?
        } else {
            0u128
        };

        let share_from_b = if pool_balance_b > 0 {
            (amount_b as u128)
                .checked_mul(pool.total_lp_issued as u128)
                .ok_or(CLMMError::ArithmeticOverflow)?
                .checked_div(pool_balance_b as u128)
                .ok_or(CLMMError::ArithmeticOverflow)?
        } else {
            0u128
        };
        std::cmp::min(share_from_a, share_from_b)
    };

    let mint_amount_u64: u64 = mint_amount.try_into().map_err(|_| CLMMError::ArithmeticOverflow)?;

    pool.total_lp_issued = pool.total_lp_issued.checked_add(mint_amount_u64).ok_or(CLMMError::ArithmeticOverflow)?;

    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            MintTo {
                mint: ctx.accounts.lp_token_mint.to_account_info(),
                to: ctx.accounts.lp_token_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info()
            }, 
            &[authority_seeds]
        ),
        mint_amount_u64
    )?;
    
    Ok(())
}