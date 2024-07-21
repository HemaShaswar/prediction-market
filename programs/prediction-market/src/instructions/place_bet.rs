use anchor_lang::prelude::*;
use anchor_spl::token::*;
use num_traits::*;

use crate::constants::*;
use crate::states::*;
use crate::MarketError;
use crate::utils::hash_to_bytes;

pub fn _place_bet(
    ctx: Context<PlaceBet>,
    bet_amount:u64,
    bet_direction: Direction,
) -> Result<()> {
    let market = &mut ctx.accounts.market;

    require!(market.initialization == MarketInitialization::InitializedPools, MarketError::InvalidMarketInitialization);

    let bet_pool: AccountInfo = match bet_direction {
        Direction::Higher => ctx.accounts.higher_pool.to_account_info(),
        Direction::Lower => ctx.accounts.lower_pool.to_account_info()
    };

    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_ata.to_account_info(),
                to: bet_pool,
                authority: ctx.accounts.user.to_account_info(),
            },
            &[&[
            market.creator.key().as_ref(), 
            &hash_to_bytes(&market.feed_id),
            &market.target_price.to_le_bytes(), 
            &market.market_duration.to_le_bytes(),
            &[ctx.accounts.market.bump],
        ]],
        ),
        bet_amount,
    )?;

    let higher_pool_amount = ctx.accounts.higher_pool.amount;
    let lower_pool_amount = ctx.accounts.lower_pool.amount;
    let odds = match bet_direction {
        Direction::Higher => {
            if lower_pool_amount == 0 {
                ODDS_FIXED_POINT_MULTIPLIER // 1.0 in fixed-point representation
            } else {
                (higher_pool_amount * ODDS_FIXED_POINT_MULTIPLIER) / lower_pool_amount
            }
        }
        Direction::Lower => {
            if higher_pool_amount == 0 {
                ODDS_FIXED_POINT_MULTIPLIER // 1.0 in fixed-point representation
            } else {
                (lower_pool_amount * ODDS_FIXED_POINT_MULTIPLIER) / higher_pool_amount
            }
        }
    };

    let bet = &mut ctx.accounts.bet;
    bet.user = ctx.accounts.user.key();
    bet.bump = ctx.bumps.bet;
    bet.amount = bet_amount;
    bet.odds =odds;
    bet.claimed = false;
    bet.market = ctx.accounts.market.key();
    bet.direction = bet_direction;
    bet.initialized = true;

    Ok(())
}

#[derive(Accounts)]
#[instruction(bet_amount:u64,bet_direction:Direction)]
pub struct PlaceBet<'info> {
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
    pub market: Account<'info, Market>,

    #[account(
        mut,
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

    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        init,
        payer = user,
        space = 8 + Bet::INIT_SPACE,
        seeds = [
            BET_SEED.as_bytes(),
            user.key().as_ref(),
            market.key().as_ref(),
            bet_amount.to_le_bytes().as_ref(),
            &bet_direction.to_u8().unwrap().to_le_bytes(),
        ], // I realize that a users may need to place the same exact bet multiple using a Bet Id might solve that
        bump
    )]
    pub bet: Account<'info,Bet>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}
