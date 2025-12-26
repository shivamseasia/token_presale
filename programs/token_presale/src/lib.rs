use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;
pub mod constants;

use instructions::*;

declare_id!("uz3pV4j51aJQ88mSsh5VasZU6YLTepAgPK2Dgs3GpZr");

#[program]
pub mod token_presale {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        token_price_usdt: u64,
        presale_duration_secs: i64,
    ) -> Result<()> {
        initialize::initialize(ctx, token_price_usdt, presale_duration_secs)
    }

    pub fn buy_tokens(ctx: Context<BuyTokens>, usdt_amount: u64) -> Result<()> {
        buy_tokens::buy_tokens(ctx, usdt_amount)
    }

    pub fn release_from_reserve(ctx: Context<ReleaseFromReserve>, amount: u64) -> Result<()> {
        release_tokens::release_tokens(ctx, amount)
    }

    pub fn update_price(ctx: Context<UpdatePrice>, new_price: u64) -> Result<()> {
        update_price::update_price(ctx, new_price)
    }
}
