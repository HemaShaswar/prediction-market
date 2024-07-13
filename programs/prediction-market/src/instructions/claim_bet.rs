use anchor_lang::prelude::*;
use anchor_spl::token::*;
use num_traits::*;

use crate::{error::MarketError, Bet, Direction, Market, BET_SEED, HIGHER_POOL_SEED, LOWER_POOL_SEED};

pub fn _claim_bet(
    ctx: Context<ClaimBet>,
) -> Result<()> {
    let bet = &mut ctx.accounts.bet;
    let market = &ctx.accounts.market;
    let clock = Clock::get()?;

    require_keys_eq!(ctx.accounts.user.key(),bet.user,MarketError::UnauthorizedUser);
    require_gt!(clock.slot,market.start_time + market.market_duration,MarketError::MarketDurationNotOver);
    require_eq!(bet.claimed,false,MarketError::BetIsClaimed);

    let bet_pool: AccountInfo = match bet.direction {
        Direction::Higher => ctx.accounts.higher_pool.to_account_info(),
        Direction::Lower => ctx.accounts.lower_pool.to_account_info()
    };

    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: bet_pool,
                to: ctx.accounts.user_ata.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        bet.amount,
    )?;

    //just for increased redundancy because the bet account should be closed after
    bet.amount = 0;
    bet.claimed = true;
    bet.initialized = false;

    Ok(())
}

#[derive(Accounts)]
pub struct ClaimBet<'info> {
    #[account(
        seeds = [
            market.creator.key().as_ref(), 
            &market.feed_id,
            &market.target_price.to_le_bytes(), 
            &market.market_duration.to_le_bytes(),
        ],
        bump = market.bump,
        address = bet.market, //check that this is the same market the bet was initialized with
    )]
    pub market: Account<'info, Market>,

    #[account(
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
        token::mint = market.mint, 
        token::authority = market,
        seeds = [
            LOWER_POOL_SEED.as_bytes(),
            market.key().as_ref()
        ],
        bump = market.lower_pool_bump,
    )]
    pub lower_pool: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = market.mint,
        associated_token::authority = user,
    )]
    pub user_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = bet.user,
    )]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        close = user,
        seeds = [
            BET_SEED.as_bytes(),
            user.key().as_ref(),
            market.key().as_ref(),
            bet.amount.to_le_bytes().as_ref(),
            &bet.direction.to_u8().unwrap().to_le_bytes(),
        ], 
        bump = bet.bump,
    )]
    pub bet: Account<'info,Bet>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}
