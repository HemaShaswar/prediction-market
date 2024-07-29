use anchor_lang::prelude::*;
use anchor_spl::token::{close_account, CloseAccount, Token, TokenAccount,transfer,Transfer};

use crate::constants::*;
use crate::states::*;
use crate::MarketError;
use crate::utils::hash_to_bytes;


pub fn _finalize_market(
    ctx: Context<FinalizeMarket>,
) -> Result<()> {
    let market = &ctx.accounts.market;
    let _clock = Clock::get()?;
    let higher_pool = &mut ctx.accounts.higher_pool;
    let lower_pool = &mut ctx.accounts.lower_pool;

    require!(market.initialization == MarketInitialization::InitializedPools,MarketError::InvalidMarketInitialization);
    require_keys_eq!(ctx.accounts.market_creator.key(),market.creator,MarketError::UnauthorizedUser);
    // require_gt!(clock.slot,market.start_time + market.market_duration + MARKET_LOCK_PERIOD,MarketError::MarketLockPeriodNotOver);
    require_eq!(higher_pool.amount + lower_pool.amount,INITIAL_USDC_POOL_AMOUNT*2,MarketError::NonZeroPools);


    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: higher_pool.to_account_info(),
                to: ctx.accounts.creator_ata.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            },
            &[&[
            market.creator.key().as_ref(), 
            &hash_to_bytes(&market.feed_id),
            &market.target_price.to_le_bytes(), 
            &market.market_duration.to_le_bytes(),
            &[ctx.accounts.market.bump],
        ]],
        ),
        INITIAL_USDC_POOL_AMOUNT,
    )?;

    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: lower_pool.to_account_info(),
                to: ctx.accounts.creator_ata.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            },
            &[&[
            market.creator.key().as_ref(), 
            &hash_to_bytes(&market.feed_id),
            &market.target_price.to_le_bytes(), 
            &market.market_duration.to_le_bytes(),
            &[ctx.accounts.market.bump],
        ]],
        ),
        INITIAL_USDC_POOL_AMOUNT,
    )?;

    close_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(), 
        CloseAccount{ 
            account: ctx.accounts.lower_pool.to_account_info(), 
            destination: ctx.accounts.market_creator.to_account_info(), 
            authority: ctx.accounts.market.to_account_info()
        }, 
        &[&[
            market.creator.key().as_ref(), 
            &hash_to_bytes(&market.feed_id),
            &market.target_price.to_le_bytes(), 
            &market.market_duration.to_le_bytes(),
            &[ctx.accounts.market.bump],
        ]],
    ))?;

    close_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(), 
        CloseAccount{ 
            account: ctx.accounts.higher_pool.to_account_info(), 
            destination: ctx.accounts.market_creator.to_account_info(), 
            authority: ctx.accounts.market.to_account_info()
        }, 
        &[&[
            market.creator.key().as_ref(), 
            &hash_to_bytes(&market.feed_id),
            &market.target_price.to_le_bytes(), 
            &market.market_duration.to_le_bytes(),
            &[ctx.accounts.market.bump],
        ]],
    ))?;

    Ok(())
}

#[derive(Accounts)]
pub struct FinalizeMarket<'info> {
    #[account(
        mut,
        close = market_creator,
        seeds = [
            market.creator.key().as_ref(), 
            &hash_to_bytes(&market.feed_id),
            &market.target_price.to_le_bytes(), 
            &market.market_duration.to_le_bytes(),
        ],
        bump = market.bump,
    )]
    pub market: Account<'info, Market>,

    #[account(
        mut,
        close = market_creator,
        token::mint = market.mint, 
        token::authority = market,
        seeds = [
            HIGHER_POOL_SEED.as_bytes(),
            market.key().as_ref(), 
        ],
        bump = market.higher_pool_bump,
    )]
    pub higher_pool: Account<'info, TokenAccount>,

    #[account(
        mut,
        close = market_creator,
        token::mint = market.mint, 
        token::authority = market,
        seeds = [
            LOWER_POOL_SEED.as_bytes(),
            market.key().as_ref(),
        ],
        bump = market.lower_pool_bump,
    )]
    pub lower_pool: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = market.mint,
        associated_token::authority = market_creator,
    )]
    pub creator_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = market.creator,
    )]
    pub market_creator: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,

}
