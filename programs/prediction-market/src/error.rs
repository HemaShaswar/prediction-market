use anchor_lang::prelude::*;

#[error_code]
pub enum MarketError {
    #[msg("Pyth Solana Feed ID is expected to have 66 characters")]
    IncorrectFeedIDLength,
    #[msg("Market Duration Can not be less than 1200 slots")]
    ShortMarketDuration,
    #[msg("Only the account creator can change account state")]
    UnauthorizedUser,
    #[msg("Market Duration is Over. Market is Locked")]
    MarketDurationOver,
    #[msg("Bet is already claimed before")]
    BetIsClaimed,
    #[msg("Market duration is ot over yet")]
    MarketDurationNotOver,
    #[msg("You can Finalize market after the MARKET_LOCK_PERIOD is over")]
    MarketLockPeriodNotOver,
}
