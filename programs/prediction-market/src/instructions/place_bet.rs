use anchor_lang::prelude::*;
use anchor_spl::token::*;
use num_traits::*;

use crate::{Bet, Direction,Market,BET_SEED, HIGHER_POOL_SEED, LOWER_POOL_SEED};

pub fn _place_bet(
    ctx: Context<PlaceBet>,
    bet_amount:u64,
    bet_direction: Direction,
) -> Result<()> {

    let bet_pool: AccountInfo = match bet_direction {
        Direction::Higher => ctx.accounts.higher_pool.to_account_info(),
        Direction::Lower => ctx.accounts.lower_pool.to_account_info()
    };

    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_ata.to_account_info(),
                to: bet_pool,
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        bet_amount,
    )?;

    let bet = &mut ctx.accounts.bet;
    bet.user = ctx.accounts.user.key();
    bet.bump = ctx.bumps.bet;
    bet.amount = bet_amount;
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
        seeds = [
            market.creator.key().as_ref(), 
            &market.feed_id,
            &market.target_price.to_le_bytes(), 
            &market.market_duration.to_le_bytes(),
        ],
        bump = market.bump,
        address = bet.market,
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
