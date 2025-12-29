use anchor_lang::prelude::*;
use crate::state::PresaleState;

#[derive(Accounts)]
pub struct Unpause<'info> {
    #[account(
        mut,
        has_one = admin
    )]
    pub state: Account<'info, PresaleState>,

    pub admin: Signer<'info>,
}

pub fn unpause(ctx: Context<Unpause>) -> Result<()> {
    ctx.accounts.state.paused = false;
    Ok(())
}
