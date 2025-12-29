use anchor_lang::prelude::*;

#[error_code]
pub enum PresaleError {
    #[msg("Presale has ended")]
    PresaleEnded,

    #[msg("Insufficient presale supply")]
    InsufficientPresaleSupply,

    #[msg("Insufficient unlocked reserve")]
    InsufficientUnlockedReserve,

    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Invalid amount")]
    InvalidAmount,

    #[msg("Wallet purchase limit exceeded")]
    WalletLimitExceeded,

    #[msg("Purchase below minimum")]
    BelowMinimum,

    #[msg("Math overflow")]
    Overflow,

    #[msg("Invalid price")]
    InvalidPrice,

    #[msg("Insufficient treasury balance")]
    InsufficientTreasuryBalance,

    #[msg("Presale is paused")]
    PresalePaused,

    #[msg("Daily withdraw limit exceeded")]
    DailyWithdrawLimitExceeded,
}
