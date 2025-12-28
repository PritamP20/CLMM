use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::{
    state::{Pool, Tick},
    utils::tick_to_sqrt_price_x64
};

#[derive(Accounts)]
#[instruction(tick_index:i32)]
pub struct InitialiseTick<'info>{
    #[account(mut)]
    pub owner:Signer<'info>,
    pub mint_a:Account<'info, Mint>,
    pub mint_b:Account<'info, Mint>,

    #[account(
        seeds=[b"authority",mint_a.key().as_ref(),mint_b.key().as_ref()],
        bump
    )]
    pub pool:AccountLoader<'info, Pool>,

    #[account(
        init,
        payer=owner,
        space=8+16+16+4+1,
        seeds=[b"tick", pool.key().as_ref(), &tick_index.to_le_bytes()],
        bump
    )]
    pub tick:Account<'info, Tick>,
    pub system_program:Program<'info, System>

}



