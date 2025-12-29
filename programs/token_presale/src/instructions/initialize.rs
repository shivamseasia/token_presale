use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::spl_token::instruction::AuthorityType;
use anchor_spl::token::{self, Mint, MintTo, SetAuthority, Token, TokenAccount};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub token_mint: Account<'info, Mint>,

    /// USDT mint (hardcode on frontend)
    pub usdt_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        seeds = [b"state"],
        bump,
        space = 8 + std::mem::size_of::<PresaleState>()
    )]
    pub state: Account<'info, PresaleState>,

    #[account(
        mut,
        constraint = vault.mint == token_mint.key(),
        constraint = vault.owner == state.key()
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = treasury.mint == usdt_mint.key(),
        constraint = treasury.owner == admin.key()
    )]
    pub treasury: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn initialize(
    ctx: Context<Initialize>,
    price_usdt: u64,
    presale_duration_secs: i64,
    daily_withdraw_limit: u64,
) -> Result<()> {
    let total_supply = 100_000_000u64 * 1_000_000;
    let presale_supply = total_supply / 10;
    let reserved_supply = total_supply - presale_supply;

    // Mint ALL tokens to vault
    token::mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.admin.to_account_info(),
            },
        ),
        total_supply,
    )?;

    // Revoke mint authority forever
    token::set_authority(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                account_or_mint: ctx.accounts.token_mint.to_account_info(),
                current_authority: ctx.accounts.admin.to_account_info(),
            },
        ),
        AuthorityType::MintTokens,
        None,
    )?;

    let state = &mut ctx.accounts.state;
    state.admin = ctx.accounts.admin.key();
    state.token_mint = ctx.accounts.token_mint.key();
    state.usdt_mint = ctx.accounts.usdt_mint.key();
    state.vault = ctx.accounts.vault.key();
    state.treasury = ctx.accounts.treasury.key();

    state.total_supply = total_supply;
    state.presale_supply = presale_supply;
    state.reserved_supply = reserved_supply;
    state.released_from_reserve = 0;

    state.daily_withdraw_limit = daily_withdraw_limit;
    state.withdrawn_today = 0;
    state.last_withdraw_ts = Clock::get()?.unix_timestamp;

    state.token_price_usdt = price_usdt;
    state.presale_end_ts = Clock::get()?.unix_timestamp + presale_duration_secs;
    state.bump = ctx.bumps.state;

    Ok(())
}
