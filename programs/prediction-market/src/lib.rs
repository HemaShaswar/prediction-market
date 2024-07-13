pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use error::*;
pub use instructions::*;
pub use state::*;

declare_id!("BKDyQffK2XZom8Fg7VgLSekNPrpCiA4TP7Yeqq1gTBty");

#[program]
pub mod prediction_market {
    use super::*;

    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        taget_price: u64,
        feed_id: String, // from https://pyth.network/developers/price-feed-ids#solana-stables
        market_duration: u64,
    ) -> Result<()> {
        _initialize_market(ctx, taget_price, feed_id, market_duration)
    }

    pub fn cancel_market(ctx: Context<CancelMarket>) -> Result<()> {
        _cancel_market(ctx)
    }

    pub fn finalize_market(ctx: Context<FinalizeMarket>) -> Result<()> {
        _finalize_market(ctx)
    }

    pub fn place_bet(
        ctx: Context<PlaceBet>,
        bet_amount: u64,
        bet_direction: Direction,
    ) -> Result<()> {
        _place_bet(ctx, bet_amount, bet_direction)
    }

    pub fn cancel_bet(ctx: Context<CancelBet>) -> Result<()> {
        _cancel_bet(ctx)
    }

    pub fn claim_bet(ctx: Context<ClaimBet>) -> Result<()> {
        _claim_bet(ctx)
    }
}
