use anchor_lang::prelude::*;
use anchor_spl::token::*;

use crate::constants::*;
use crate::states::*;
use crate::MarketError;
use crate::utils::hash_to_bytes;

pub fn _initialize_pools(
    ctx: Context<InitializePools>,
) -> Result<()> {
    let market = &mut ctx.accounts.market;
    require!(market.initialization == MarketInitialization::InitializedMarket,MarketError::InvalidMarketInitialization);
    // require_eq!(
    //     ctx.accounts.pool_token_mint.key().to_string(),
    //     USDC_MINT, //more tokens can be allowed in the future
    //     MarketError::InvalidPoolMint
    // ); 
    
    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_ata.to_account_info(),
                to: ctx.accounts.higher_pool.to_account_info(),
                authority: ctx.accounts.market_creator.to_account_info(),
            },
        ),
        INITIAL_USDC_POOL_AMOUNT,
    )?;

    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_ata.to_account_info(),
                to: ctx.accounts.lower_pool.to_account_info(),
                authority: ctx.accounts.market_creator.to_account_info(),
            },
        ),
        INITIAL_USDC_POOL_AMOUNT,
    )?;

    //Nothing after this comments seems to run at all
    
    market.mint = ctx.accounts.pool_token_mint.key();
    
    market.lower_pool_bump = ctx.bumps.lower_pool;
    market.higher_pool_bump = ctx.bumps.higher_pool;

    market.initialization = MarketInitialization::InitializedPools;

    Ok(())
}

#[derive(Accounts)]
pub struct InitializePools<'info> {
    #[account(
        mut,
        seeds = [
            market.creator.key().as_ref(), 
            &hash_to_bytes(&market.feed_id),
            &market.target_price.to_le_bytes(), 
            &market.market_duration.to_le_bytes(),
        ],
        bump = market.bump,
    )]
    pub market: Box<Account<'info, Market>>,

    #[account(
        init,
        payer = market_creator,
        token::mint = pool_token_mint, 
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
        token::mint = pool_token_mint, 
        token::authority = market,
        seeds = [
            LOWER_POOL_SEED.as_bytes(),
            market.key().as_ref(),
        ],
        bump
    )]
    pub lower_pool: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = pool_token_mint,
        associated_token::authority = market.creator,
    )]
    pub user_ata: Box<Account<'info, TokenAccount>>,

    //token mint account that bets are gonna be made with e.g USDC
    pub pool_token_mint: Account<'info,Mint>,

    #[account(
        mut,
        address = market.creator,
    )]
    pub market_creator: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,

}
