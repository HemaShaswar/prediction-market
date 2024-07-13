use crate::{error::MarketError, MarketState, HIGHER_POOL_SEED, LOWER_POOL_SEED};
use anchor_lang::prelude::*;
use anchor_spl::token::*;

use crate::Market;

pub fn _initialize_market(
    ctx: Context<InitializeMarket>,
    target_price: u64,
    feed_id: String, // from https://pyth.network/developers/price-feed-ids#solana-stables
    market_duration: u64,
) -> Result<()> {
    require_eq!(feed_id.len(), 66, MarketError::IncorrectFeedIDLength);
    require_gte!(market_duration, 1200, MarketError::ShortMarketDuration); //more than one hour

    let market = &mut ctx.accounts.market;
    
    let clock = Clock::get()?;
    market.start_time = clock.slot;
   
    market.target_price = target_price;
    market.market_duration = market_duration;

    let mut feed_data = [0u8;66];
    feed_data.copy_from_slice(feed_id.as_bytes());
    market.feed_id = feed_data;
    
    
    market.creator = ctx.accounts.market_creator.key();
    market.mint = ctx.accounts.market_mint_account.key();
    
    market.bump = ctx.bumps.market;
    market.lower_pool_bump = ctx.bumps.lower_pool;
    market.higher_pool_bump = ctx.bumps.higher_pool;

    market.state = MarketState::Initialized;

    Ok(())
}

#[derive(Accounts)]
#[instruction(target_price:u64,feed_id:String,market_duration:u64)]
pub struct InitializeMarket<'info> {
    #[account(
        init,
        payer = market_creator,
        space = 8 + Market::INIT_SPACE,
        seeds = [
            market_creator.key().as_ref(), 
            feed_id.as_bytes(),
            &target_price.to_le_bytes(), 
            &market_duration.to_le_bytes(),
        ],
        bump
    )]
    pub market: Account<'info, Market>,

    #[account(
        init,
        payer = market_creator,
        token::mint = market_mint_account, 
        token::authority = market,
        seeds = [
            HIGHER_POOL_SEED.as_bytes(),
            market.key().as_ref(), 
        ],
        bump
    )]
    pub higher_pool: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = market_creator,
        token::mint = market_mint_account, 
        token::authority = market,
        seeds = [
            LOWER_POOL_SEED.as_bytes(),
            market.key().as_ref(),
        ],
        bump
    )]
    pub lower_pool: Account<'info, TokenAccount>,

    //token mint account that bets are gonna be made with e.g JUP
    pub market_mint_account: Account<'info,Mint>,

    #[account(mut)]
    pub market_creator: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,

}
