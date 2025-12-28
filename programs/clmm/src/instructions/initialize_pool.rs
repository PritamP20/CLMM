use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, TokenAccount, Token};

use crate::utils::{price_to_sqrt_price_x64, sqrt_price_x64_to_tick};
use crate::state::Pool;

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
