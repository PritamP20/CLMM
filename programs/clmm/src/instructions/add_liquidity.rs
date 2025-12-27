use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;

#[derive(Accounts)]
#[instruction(tick_lower:i32,tick_upper:i32)]
pub struct AddLiquidity<'info>{
    #[account(mut)]
    pub signer:Signer<'info>,

    #[account(seeds=[b"authority", mint_a.key().as_ref(), min_b.key().as_ref()], bump)]
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
        bumb
    )]
    pub tick_lower:Account<'info, Tick>,
    #[account(
        mut,
        seeds=[b"tick", mint_a.key().as_ref(), mint_b.key().as_ref(), &tick_upper.to_le_bytes()],
        bump
    )]
    pub tick_upper:Account<'info, Mint>,
    pub mint_a:Account<'info, Mint>,
    pub mint_b:Account<'info, Mint>,

    #[account(
        mut,
        seeds=[b"vault_token", mint_a.key().as_ref(), mint_b.key().as_ref(),b"A"],
        bumb,
        token::mint=mint_a,
        token::authority=authority
    )]
    pub vault_a:Box<Account<'info, TokenAccount>>,
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
        seeds=[b"vault_token", mint_a.key().as_ref(), mint_b,key(), b"B"]
        bump,
        token::mint=mint_b,
        token::authority=authority
    )]
    pub vault_b:Box<Account<'info, TokenAccount>>, //what ist he difference in Box<Accoiunt>

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
    require!(tick_lower<tick_upper, CLMMError::TickMismatch)
}