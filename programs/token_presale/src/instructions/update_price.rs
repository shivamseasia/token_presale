use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct UpdatePrice<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut, seeds = [b"state"], bump = state.bump)]
    pub state: Account<'info, PresaleState>,
}

pub fn handler(ctx: Context<UpdatePrice>, new_price: u64) -> Result<()> {
    let state = &mut ctx.accounts.state;

    require!(ctx.accounts.admin.key() == state.admin, PresaleError::Unauthorized);

    state.token_price_usdt = new_price;

    Ok(())
}
