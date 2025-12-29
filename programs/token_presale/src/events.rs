 use anchor_lang::prelude::*;

#[event]
pub struct TokensPurchased {
    pub buyer: Pubkey,
    pub usdt_amount: u64,
    pub tokens_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct UsdtWithdrawn {
    pub admin: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct PresalePaused {
    pub admin: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct PresaleUnpaused {
    pub admin: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct PriceUpdated {
    pub admin: Pubkey,
    pub old_price: u64,
    pub new_price: u64,
    pub timestamp: i64,
}

#[event]
pub struct DailyWithdraw {
    pub admin: Pubkey,
    pub amount: u64,
    pub withdrawn_today: u64,
    pub timestamp: i64,
}