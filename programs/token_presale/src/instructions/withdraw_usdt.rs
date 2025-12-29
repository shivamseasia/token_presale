use crate::events::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::errors::PresaleError;
use crate::state::PresaleState;

#[derive(Accounts)]
pub struct WithdrawUsdt<'info> {
    #[account(mut, has_one = admin)]
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

    let now = Clock::get()?.unix_timestamp;
    let state = &mut ctx.accounts.state;

    // Reset after 24 hours
    if now - state.last_withdraw_ts >= 86_400 {
        state.withdrawn_today = 0;
        state.last_withdraw_ts = now;
    }

    // Enforce daily limit
    require!(
        state.withdrawn_today + amount <= state.daily_withdraw_limit,
        PresaleError::DailyWithdrawLimitExceeded
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

    state.withdrawn_today = state
        .withdrawn_today
        .checked_add(amount)
        .ok_or(PresaleError::Overflow)?;

    emit!(UsdtWithdrawn {
        admin: ctx.accounts.admin.key(),
        amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    emit!(DailyWithdraw {
        admin: ctx.accounts.admin.key(),
        amount,
        withdrawn_today: state.withdrawn_today,
        timestamp: now,
    });

    Ok(())
}
