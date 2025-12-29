use crate::constants::*;
use crate::{
    errors::PresaleError,
    state::{PresaleState, UserPurchase},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Transfer};

#[derive(Accounts)]
pub struct BuyTokens<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// Tracks lifetime purchases per wallet
    #[account(
        init_if_needed,
        payer = buyer,
        seeds = [b"user_purchase", buyer.key().as_ref()],
        bump,
        space = 8 + std::mem::size_of::<UserPurchase>()
    )]
    pub user_purchase: Account<'info, UserPurchase>,

    /// Buyer's USDT account (stack-safe)
    #[account(mut)]
    pub buyer_usdt: AccountInfo<'info>,

    /// Treasury USDT account (admin-owned)
    #[account(mut)]
    pub treasury_usdt: AccountInfo<'info>,

    /// Program vault holding sale tokens
    #[account(mut)]
    pub vault: AccountInfo<'info>,

    /// Buyer's token account
    #[account(mut)]
    pub buyer_token: AccountInfo<'info>,

    /// Global presale state PDA
    #[account(
        mut,
        seeds = [b"state"],
        bump = state.bump
    )]
    pub state: Account<'info, PresaleState>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn buy_tokens(ctx: Context<BuyTokens>, usdt_amount: u64) -> Result<()> {
    require!(!ctx.accounts.state.paused, PresaleError::PresalePaused);
    require!(usdt_amount > 0, PresaleError::InvalidAmount);

    let state = &mut ctx.accounts.state;
    let user = &mut ctx.accounts.user_purchase;
    let now = Clock::get()?.unix_timestamp;

    let tokens = (usdt_amount as u128)
        .checked_mul(TOKEN_DECIMALS as u128)
        .ok_or(PresaleError::Overflow)?
        .checked_div(state.token_price_usdt as u128)
        .ok_or(PresaleError::InvalidPrice)? as u64;

    require!(tokens >= MIN_TOKENS_PER_BUY, PresaleError::BelowMinimum);

    if user.total_bought == 0 {
        user.buyer = ctx.accounts.buyer.key();
        user.bump = ctx.bumps.user_purchase;
    }

    let new_total = user
        .total_bought
        .checked_add(tokens)
        .ok_or(PresaleError::Overflow)?;

    require!(
        new_total <= MAX_TOKENS_PER_WALLET,
        PresaleError::WalletLimitExceeded
    );

    user.total_bought = new_total;

    if now <= state.presale_end_ts {
        require!(
            state.presale_supply >= tokens,
            PresaleError::InsufficientPresaleSupply
        );
        state.presale_supply -= tokens;
    } else {
        require!(
            state.released_from_reserve >= tokens,
            PresaleError::InsufficientUnlockedReserve
        );
        state.released_from_reserve -= tokens;
    }

    // Transfer USDT from buyer to treasury
    let cpi_ctx_usdt = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.buyer_usdt.clone(),
            to: ctx.accounts.treasury_usdt.clone(),
            authority: ctx.accounts.buyer.to_account_info().clone(),
        },
    );
    token::transfer(cpi_ctx_usdt, usdt_amount)?;

    let seeds: &[&[u8]] = &[b"state".as_ref(), &[state.bump]];
    let signer = &[seeds];

    // Transfer sale tokens from vault to buyer
    let cpi_ctx_token = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault.clone(),
            to: ctx.accounts.buyer_token.clone(),
            authority: ctx.accounts.state.to_account_info().clone(),
        },
        signer,
    );
    token::transfer(cpi_ctx_token, tokens)?;

    Ok(())
}
