use anchor_lang::prelude::*;

#[constant]
pub const LOWER_POOL_SEED: &str = "lower_pool";
#[constant]
pub const HIGHER_POOL_SEED: &str = "higher_pool";
#[constant]
pub const BET_SEED: &str = "prediction_bet";
#[constant]
pub const MARKET_LOCK_PERIOD: u64 = 576000; //more than two days
#[constant]
pub const USDC_MINT: &str = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU";
#[constant]
pub const INITIAL_USDC_POOL_AMOUNT: u64 = 1000000;
#[constant]
pub const ODDS_FIXED_POINT_MULTIPLIER: u64 = 1_000_000;
