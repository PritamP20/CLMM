use anchor_lang::prelude::*;
mod state;
mod utils;
declare_id!("G6kwUoHSqYmzewHR1npTFa3LncPmrksNkJ99Cyc8JJPz");

#[program]
pub mod clmm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
