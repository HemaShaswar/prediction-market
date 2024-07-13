use anchor_lang::prelude::*;

#[constant]
pub const LOWER_POOL_SEED: &str = "lower_pool";
pub const HIGHER_POOL_SEED: &str = "higher_pool";
pub const BET_SEED: &str = "prediction_bet";
pub const MARKET_LOCK_PERIOD: u64 = 576000; //more than two days
