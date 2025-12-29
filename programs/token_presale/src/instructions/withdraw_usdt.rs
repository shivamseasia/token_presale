use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::errors::PresaleError;
use crate::state::PresaleState;

#[derive(Accounts)]
pub struct WithdrawUsdt<'info> {
    #[account(
        has_one = admin,
    )]
    pub state: Account<'info, PresaleState>,

    pub admin: Signer<'info>,

    #[account(
        mut,
        constraint = treasury.key() == state.treasury,
        constraint = treasury.owner == admin.key(),
    )]
    pub treasury: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = destination.mint == treasury.mint,
        constraint = destination.owner == admin.key(),
    )]
    pub destination: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn withdraw_usdt(ctx: Context<WithdrawUsdt>, amount: u64) -> Result<()> {
    require!(amount > 0, PresaleError::InvalidAmount);
    require!(
        ctx.accounts.treasury.amount >= amount,
        PresaleError::InsufficientTreasuryBalance
    );

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.treasury.to_account_info(),
                to: ctx.accounts.destination.to_account_info(),
                authority: ctx.accounts.admin.to_account_info(),
            },
        ),
        amount,
    )?;

    Ok(())
}
