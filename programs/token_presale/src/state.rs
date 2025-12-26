use anchor_lang::prelude::*;

#[account]
pub struct PresaleState {
    pub admin: Pubkey,

    pub token_mint: Pubkey,
    pub usdt_mint: Pubkey,

    pub vault: Pubkey,
    pub treasury: Pubkey,

    pub total_supply: u64,
    pub presale_supply: u64,
    pub reserved_supply: u64,

    pub released_from_reserve: u64,

    pub token_price_usdt: u64,
    pub presale_end_ts: i64,

    pub bump: u8,
}

#[account]
pub struct UserPurchase {
    pub buyer: Pubkey,
    pub total_bought: u64, // in smallest token units
    pub bump: u8,
}

