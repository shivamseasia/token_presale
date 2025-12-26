use anchor_lang::prelude::*;

#[constant]
pub const TOKEN_DECIMALS: u64 = 1_000_000;             // Token decimals
pub const MAX_TOKENS_PER_WALLET: u64 = 50_000 * TOKEN_DECIMALS; // Lifetime cap per wallet
pub const MIN_TOKENS_PER_BUY: u64 = 10 * TOKEN_DECIMALS;