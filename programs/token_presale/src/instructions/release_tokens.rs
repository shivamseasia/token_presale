use anchor_lang::prelude::*;
use crate::{state::*, errors::*};

#[derive(Accounts)]
pub struct ReleaseFromReserve<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state"],
        bump = state.bump
    )]
    pub state: Account<'info, PresaleState>,
}

pub fn release_tokens(
    ctx: Context<ReleaseFromReserve>,
    amount: u64,
) -> Result<()> {

    let state = &mut ctx.accounts.state;
    let now = Clock::get()?.unix_timestamp;

    require!(ctx.accounts.admin.key() == state.admin, PresaleError::Unauthorized);
    require!(now > state.presale_end_ts, PresaleError::PresaleEnded);
    require!(state.reserved_supply >= amount, PresaleError::InsufficientUnlockedReserve);

    state.reserved_supply -= amount;
    state.released_from_reserve += amount;

    Ok(())
}
