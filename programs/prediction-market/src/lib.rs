pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("B1PbzLhQLyyhZu39rzcCJ7W1UGujTMjrTpcsmBb6K7rT");

#[program]
pub mod prediction_market {
    use super::*;

    pub fn initialize(
        ctx: Context<InitializeMarket>,
        taget_price: u64,
        feed_id: String, // from https://pyth.network/developers/price-feed-ids#solana-stables
        market_duration: u64,
    ) -> Result<()> {
        _initialize_market(ctx, taget_price, feed_id, market_duration)
    }

    pub fn place_bet(ctx: Context<PlaceBet>, bet_amount: u64) -> Result<()> {
        Ok(())
    }

    pub fn cancel_bet(ctx: Context<PlaceBet>, bet_amount: u64) -> Result<()> {
        _cancel_bet();
        Ok(())
    }
}
