use anchor_lang::prelude::*;
use crate::state::PresaleState;
use crate::events::*;

#[derive(Accounts)]
pub struct Pause<'info> {
    #[account(
        mut,
        has_one = admin
    )]
    pub state: Account<'info, PresaleState>,

    pub admin: Signer<'info>,
}

pub fn pause(ctx: Context<Pause>) -> Result<()> {
    ctx.accounts.state.paused = true;
    emit!(PresalePaused {
        admin: ctx.accounts.admin.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}
