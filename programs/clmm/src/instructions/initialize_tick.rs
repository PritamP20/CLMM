use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::{
    state::{Pool, Tick},
    utils::tick_to_sqrt_price_x64
};

pub fn initialize_tick(
    ctx:Context<InitialiseTick>,
    tick_index:i32
)->Result<()>{
    let tick = &mut ctx.accounts.tick;
    let sqrt_price_x64 = tick_to_sqrt_price_x64(tick_index)?;
    tick.index = tick_index;
    tick.squrt_price_x64 = sqrt_price_x64;
    tick.liquidity_net = 0;
    tick.bump = ctx.bumps.tick;
    Ok(())
}

#[derive(Accounts)]
#[instruction(tick_index:i32)]
pub struct InitialiseTick<'info>{
    #[account(mut)]
    pub owner:Signer<'info>,
    pub mint_a:Account<'info, Mint>,
    pub mint_b:Account<'info, Mint>,

    #[account(
        seeds=[b"pool",mint_a.key().as_ref(),mint_b.key().as_ref()],
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



