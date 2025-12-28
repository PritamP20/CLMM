use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, TokenAccount, Token};

use crate::utils::{price_to_sqrt_price_x64, sqrt_price_x64_to_tick};
use crate::state::Pool;

pub fn initialize_pool(
    ctx:Context<InitializePool>,
    current_price:u64
)->Result<()>{
    let curr_sqrt_price_x64=price_to_sqrt_price_x64(current_price)?;
    let current_tick = sqrt_price_x64_to_tick(curr_sqrt_price_x64)?;

    let mut pool = ctx.accounts.pool.load_init()?;

    pool.mint_a=ctx.accounts.mint_a.key();
    pool.mint_b=ctx.accounts.mint_b.key();
    pool.vault_a = ctx.accounts.vault_a.key();
    pool.vault_b = ctx.accounts.vault_b.key();
    pool.lp_mint= ctx.accounts.lp_token_mint.key();

    pool.total_lp_issued = 0;
    pool.sqrt_price_x64=curr_sqrt_price_x64;
    pool.current_tick = current_tick;
    pool.active_liquidity = 0;
    Ok(())  
}

#[derive(Accounts)]
pub struct InitializePool<'info>{
    #[account(mut)]
    pub owner:Signer<'info>,

    #[account(
        seeds=[b"authority", mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump
    )]
    pub authority:UncheckedAccount<'info>,

    pub mint_a:Account<'info, Mint>,
    pub mint_b:Account<'info, Mint>,

    #[account(
        init,
        payer=owner,
        associated_token::mint=mint_a,
        associated_token::authority=authority
    )]
    pub vault_a:Account<'info, TokenAccount>,

    #[account(
        init,
        payer=owner,
        associated_token::mint=mint_b,
        associated_token::authority=authority
    )]
    pub vault_b:Account<'info, TokenAccount>,

    #[account(
        init,
        payer=owner,
        seeds=[b"lp_mint", mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump,
        mint::decimals=6,
        mint::authority=authority,
        mint::freeze_authority=authority
    )]
    pub lp_token_mint:Account<'info, Mint>,

    #[account(
        init,
        payer=owner,
        space=8+32*6+16*2+8+4+1+3,
        seeds=[b"pool", mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump
    )]
    pub pool:AccountLoader<'info, Pool>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program:Program<'info, AssociatedToken>,
    pub system_program:Program<'info, System>

}
