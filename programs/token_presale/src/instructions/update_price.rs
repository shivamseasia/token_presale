use anchor_lang::prelude::*;
use crate::errors::*;
use crate::state::*;
use crate::events::*;

#[derive(Accounts)]
pub struct UpdatePrice<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut, seeds = [b"state"], bump = state.bump)]
    pub state: Account<'info, PresaleState>,
}

pub fn update_price(ctx: Context<UpdatePrice>, new_price: u64) -> Result<()> {
    require!(!ctx.accounts.state.paused, PresaleError::PresalePaused);
    let state = &mut ctx.accounts.state;
    require!(
        ctx.accounts.admin.key() == state.admin,
        PresaleError::Unauthorized
    );

    let old_price = state.token_price_usdt;
    state.token_price_usdt = new_price;
    emit!(PriceUpdated {
        admin: ctx.accounts.admin.key(),
        old_price,
        new_price,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
