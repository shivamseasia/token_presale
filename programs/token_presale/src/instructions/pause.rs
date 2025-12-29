use anchor_lang::prelude::*;
use crate::state::PresaleState;

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
    Ok(())
}
