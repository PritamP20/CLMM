use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{burn,transfer,Burn,Mint,Token,TokenAccount,Transfer}};

use crate::{state::{Pool,Tick}, utils::{calculate_liquidity_amounts, tick_to_sqrt_price_x64}};
use crate::error::*;


#[derive(Accounts)]
#[instruction(tick_lower:i32,tick_upper:i32)]
pub struct Withdraw<'info>{
    #[account(mut)]
    pub liquidity_provider:Signer<'info>,

    pub mint_a:Account<'info,Mint>,
    pub mint_b:Account<'info,Mint>,
    
    /// CHECK:test
    #[account(
        seeds=[b"authority",mint_a.key().as_ref(),mint_b.key().as_ref()],
        bump
    )]
    pub authority:UncheckedAccount<'info>,

    #[account(
        mut,
        seeds=[b"pool",mint_a.key().as_ref(),mint_b.key().as_ref()],
        bump
    )]
    pub pool:AccountLoader<'info,Pool>,

    #[account(
        mut,
        seeds=[b"tick",pool.key().as_ref(),tick_lower.index.to_le_bytes().as_ref()],
        bump
    )]
    pub tick_lower:Account<'info,Tick>,

    #[account(
        mut,
        seeds=[b"tick",pool.key().as_ref(),tick_upper.index.to_le_bytes().as_ref()],
        bump
    )]
    pub tick_upper:Account<'info,Tick>,

    #[account(
        mut,
        associated_token::mint=mint_a,
        associated_token::authority=liquidity_provider
    )]
    pub token_account_a:Account<'info,TokenAccount>,

    #[account(
        mut,
        associated_token::mint=mint_b,
        associated_token::authority=liquidity_provider
    )]
    pub token_account_b:Account<'info,TokenAccount>,

    #[account(
        mut,
        seeds=[b"lp_mint",mint_a.key().as_ref(),mint_b.key().as_ref()],
        bump
    )]
    pub lp_token_mint:Account<'info,Mint>,

    #[account(
        mut,
        associated_token::mint=lp_token_mint,
        associated_token::authority=liquidity_provider
    )]
    pub lp_token_account:Account<'info,TokenAccount>,

    #[account(
        mut,
        seeds=[b"vault",mint_a.key().as_ref(),mint_b.key().as_ref(),b"A"],
        bump
    )]
    pub vault_a:Account<'info,TokenAccount>,

    #[account(
        mut,
        seeds=[b"vault",mint_a.key().as_ref(),mint_b.key().as_ref(),b"B"],
        bump
    )]
    pub vault_b:Account<'info,TokenAccount>,

    pub token_program:Program<'info,Token>,
    pub associated_token_program:Program<'info,AssociatedToken>,
    pub system_program:Program<'info,System>,

}


pub fn withdraw(ctx:Context<Withdraw>,tick_lower:i32,tick_upper:i32,liquidity_remove_amount:u128)->Result<()>{
    require!(tick_upper>tick_lower,CLMMError::TickMismatch);

    let mut pool=ctx.accounts.pool.load_mut()?;
    require!(liquidity_remove_amount>0,CLMMError::ZeroAmount);
    require!(pool.total_lp_issued>0,CLMMError::PoolEmpty);



    let tick_upper_acc = &mut ctx.accounts.tick_upper;
    let tick_lower_acc = &mut ctx.accounts.tick_lower;

    let sqrt_price_lower_x64=tick_to_sqrt_price_x64(tick_lower)?;
    let sqrt_price_upper_x64=tick_to_sqrt_price_x64(tick_upper)?;

    let (withdraw_amount_a,withdraw_amount_b)=calculate_liquidity_amounts(pool.sqrt_price_x64, sqrt_price_lower_x64, sqrt_price_upper_x64, liquidity_remove_amount)?;

    let pool_balance_a=ctx.accounts.vault_a.amount;
    let pool_balance_b=ctx.accounts.vault_b.amount;

    let lp_tokens_to_burn=if pool_balance_a> 0 && pool_balance_b>0{
        let share_from_a=(withdraw_amount_a as u128)
        .checked_mul(pool.total_lp_issued as u128)
        .ok_or(CLMMError::ArithmeticOverflow)?
        .checked_div(pool_balance_a as u128)
        .ok_or(CLMMError::ArithmeticOverflow)?;

        let share_from_b=(withdraw_amount_b as u128)
        .checked_mul(pool.total_lp_issued as u128)
        .ok_or(CLMMError::ArithmeticOverflow)?
        .checked_div(pool_balance_b as u128)
        .ok_or(CLMMError::ArithmeticOverflow)?;

        std::cmp::max(share_from_a,share_from_b) as u64

    }else if pool_balance_a>0{
        ((withdraw_amount_a as u128)
        .checked_mul(pool.total_lp_issued as u128)
        .ok_or(CLMMError::ArithmeticOverflow)?
        .checked_div(pool_balance_a as u128)
        .ok_or(CLMMError::ArithmeticOverflow)?) as u64
    }else if pool_balance_b>0{
         ((withdraw_amount_b as u128)
            .checked_mul(pool.total_lp_issued as u128)
            .ok_or(CLMMError::ArithmeticOverflow)?
            .checked_div(pool_balance_b as u128)
            .ok_or(CLMMError::ArithmeticOverflow)?) as u64
    }else{
        return Err(CLMMError::PoolEmpty.into());
    };

    require_eq!(tick_lower_acc.index,tick_lower,CLMMError::InvalidTickIndex);
    require_eq!(tick_upper_acc.index,tick_upper,CLMMError::InvalidTickIndex);

    require!(
        ctx.accounts.lp_token_account.amount>=lp_tokens_to_burn,
        CLMMError::InsufficientLPTokens
    );

    tick_lower_acc.liquidity_net=tick_lower_acc.liquidity_net
    .checked_sub(liquidity_remove_amount as i128)
    .ok_or(CLMMError::ArithmeticOverflow)?;


    tick_upper_acc.liquidity_net=tick_upper_acc.liquidity_net
    .checked_add(liquidity_remove_amount as i128)
    .ok_or(CLMMError::ArithmeticOverflow)?;

    if tick_lower<=pool.current_tick && pool.current_tick<=tick_upper{
        pool.active_liquidity=pool.active_liquidity
        .checked_sub(liquidity_remove_amount)
        .ok_or(CLMMError::ArithmeticOverflow)?;
    }
    
    let (amount_a,amount_b)=(withdraw_amount_a,withdraw_amount_b);
    let token_a_mint=ctx.accounts.mint_a.key();
    let token_b_mint=ctx.accounts.mint_b.key();

    let seeds:&[&[u8]]=&[b"authority",token_a_mint.as_ref(),token_b_mint.as_ref(),&[ctx.bumps.authority]];

    let signer=&[seeds];

    if amount_a!=0{
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer { from:ctx.accounts.vault_a.to_account_info(), to:ctx.accounts.token_account_a.to_account_info(), authority: ctx.accounts.authority.to_account_info() },
            
            signer,
        ),
        amount_a
    )?;
    }
    if amount_b!=0{
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer { from: ctx.accounts.vault_b.to_account_info(), to: ctx.accounts.token_account_b.to_account_info(), authority: ctx.accounts.authority.to_account_info() },
                signer, 
                ),
                amount_b,
        )?;
    }
    burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn { mint: ctx.accounts.lp_token_mint.to_account_info(), from: ctx.accounts.lp_token_account.to_account_info(), authority: ctx.accounts.liquidity_provider.to_account_info() }
        ),
        lp_tokens_to_burn as u64
    )?;

    pool.total_lp_issued=pool.total_lp_issued
    .checked_sub(lp_tokens_to_burn)
    .ok_or(CLMMError::ArithmeticOverflow)?;
    
    Ok(())
}