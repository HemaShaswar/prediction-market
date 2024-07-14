use anchor_lang::prelude::*;

#[constant]
pub const LOWER_POOL_SEED: &str = "lower_pool";
pub const HIGHER_POOL_SEED: &str = "higher_pool";
pub const BET_SEED: &str = "prediction_bet";
pub const MARKET_LOCK_PERIOD: u64 = 576000; //more than two days
pub const USDC_MINT: &str = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU";
pub const INITIAL_USDC_POOL_AMOUNT: u64 = 1000000;
pub const ODDS_FIXED_POINT_MULTIPLIER: u64 = 1_000_000;
