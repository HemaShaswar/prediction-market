use anchor_lang::prelude::*;

#[error_code]
pub enum MarketError {
    #[msg("Pyth Solana Feed ID is expected to have 66 characters")]
    IncorrectFeedIDLength,
    #[msg("Market Duration Can not be less than 1200 slots")]
    ShortMarketDuration,
    #[msg("Only the account creator can change account state")]
    UnauthorizedUser,
    #[msg("Bet is already claimed before")]
    BetIsClaimed,
    #[msg("Market key must match bet market key")]
    BetMarketMismatch,
    #[msg("Invalid market stage. Must perform instruction at correct time period")]
    InvalidMarketInitialization,
    #[msg("Market betting Duration is not over yet")]
    MarketDurationNotOver,
    #[msg("576000 slots must pass after Market betting period before closing market")]
    MarketLockPeriodNotOver,
    #[msg("Bet Can only be placed during the market duration")]
    MarketDurationOver,
    #[msg("Bet pools are non zero. Market can not cancel")]
    NonZeroPools,
    #[msg("Error during UTF8 conversion")]
    InvalidUtf8,
    #[msg("The provided mint account is not allowed")]
    InvalidPoolMint,
    #[msg("Market Final price is empty. Error during getting price")]
    NoneFinalPrice,
    #[msg("Error converting feedID to hex")]
    InvalidFeedId,
    #[msg("Overflow occured when adjusting the oracle price data")]
    PriceAdjustmentOverflow,
}
