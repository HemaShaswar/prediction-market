use anchor_lang::prelude::*;
use num_derive::*;

#[account]
#[derive(InitSpace)]
pub struct Market {
    pub creator: Pubkey,
    pub target_price: u64,
    pub start_time: u64,
    pub market_duration: u64,
    pub bump: u8,
    pub mint: Pubkey,
    pub higher_pool_bump: u8,
    pub lower_pool_bump: u8,
    pub feed_id: [u8; 66], // from https://pyth.network/developers/price-feed-ids#solana-stables
    pub initialized: bool,
}

#[derive(
    AnchorSerialize, AnchorDeserialize, Clone, InitSpace, ToPrimitive, FromPrimitive, PartialEq, Eq,
)]
pub enum Direction {
    Higher,
    Lower,
}
#[account]
#[derive(InitSpace)]
pub struct Bet {
    pub user: Pubkey,
    pub market: Pubkey,
    pub amount: u64,
    pub direction: Direction,
    pub claimed: bool,
    pub bump: u8,
    pub initialized: bool,
}
