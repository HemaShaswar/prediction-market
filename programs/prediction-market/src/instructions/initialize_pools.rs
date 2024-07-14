use crate::{Market, MarketError,MarketInitialization, HIGHER_POOL_SEED, LOWER_POOL_SEED};
use anchor_lang::prelude::*;
use anchor_spl::token::{Token,TokenAccount,Mint};

pub fn _initialize_pools(
    ctx: Context<InitializePools>,
) -> Result<()> {
    let market = &mut ctx.accounts.market;
    require!(market.initialization == MarketInitialization::InitializedMarket,MarketError::InvalidMarketInitialization);
    
    market.mint = ctx.accounts.market_mint_account.key();
    
    market.lower_pool_bump = ctx.bumps.lower_pool;
    market.higher_pool_bump = ctx.bumps.higher_pool;

    market.initialization = MarketInitialization::InitializedPools;

    Ok(())
}

#[derive(Accounts)]
pub struct InitializePools<'info> {
    #[account(
        seeds = [
            market.creator.key().as_ref(), 
            &market.feed_id,
            &market.target_price.to_le_bytes(), 
            &market.market_duration.to_le_bytes(),
        ],
        bump = market.bump,
    )]
    pub market: Box<Account<'info, Market>>,

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
    pub higher_pool: Box<Account<'info, TokenAccount>>,

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
    pub lower_pool: Box<Account<'info, TokenAccount>>,

    //token mint account that bets are gonna be made with e.g JUP
    pub market_mint_account: Box<Account<'info,Mint>>,

    #[account(
        mut,
        address = market.creator,
    )]
    pub market_creator: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,

}
