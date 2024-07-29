use anchor_lang::prelude::*;
use anchor_spl::token::*;
use num_traits::*;
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex,PriceUpdateV2};

use crate::constants::*;
use crate::states::*;
use crate::MarketError;
use crate::utils::hash_to_bytes;


pub fn _claim_bet(
    ctx: Context<ClaimBet>,
) -> Result<()> {
    let bet = &mut ctx.accounts.bet;
    let market = &mut ctx.accounts.market;
    let price_update = &ctx.accounts.price_update;
    let clock = Clock::get()?;

    require!(market.initialization == MarketInitialization::InitializedPools,MarketError::InvalidMarketInitialization);
    require_keys_eq!(market.key(),bet.market,MarketError::BetMarketMismatch);
    require_keys_eq!(ctx.accounts.user.key(),bet.user,MarketError::UnauthorizedUser);
    require_gt!(clock.slot,market.start_time + market.market_duration,MarketError::MarketDurationNotOver);
    require_eq!(bet.claimed,false,MarketError::BetIsClaimed);


    if market.final_price.is_none() {
        let feed_id_str = std::str::from_utf8(&market.feed_id)
            .map_err(|_| MarketError::InvalidUtf8)?;

        let feed_id = get_feed_id_from_hex(feed_id_str)
            .map_err(|_| MarketError::InvalidFeedId)?;

        let price =price_update.get_price_no_older_than(
            &clock,
            30_u64,
            &feed_id 
        )?;

        let adjusted_price = if price.exponent < 0 {
            (price.price as u64).checked_mul(10_u64.pow(price.exponent.abs() as u32))
                .ok_or(MarketError::PriceAdjustmentOverflow)?
        } else {
            (price.price as u64).checked_div(10_u64.pow(price.exponent as u32))
                .ok_or(MarketError::PriceAdjustmentOverflow)?
        };

        market.final_price = Some(adjusted_price);
    }

    require!(
        market.final_price.is_some(),
        MarketError::NoneFinalPrice
    );

    bet.is_won = match bet.direction {
        Direction::Higher => market.final_price.unwrap() > market.target_price,
        Direction::Lower => market.final_price.unwrap() < market.target_price,
    };
    
    if bet.is_won {
        let bet_pool: AccountInfo = match bet.direction {
            Direction::Higher => ctx.accounts.higher_pool.to_account_info(),
            Direction::Lower => ctx.accounts.lower_pool.to_account_info(),
        };

        let payout = bet.amount.checked_mul(bet.odds).unwrap() / ODDS_FIXED_POINT_MULTIPLIER;
    
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: bet_pool,
                    to: ctx.accounts.user_ata.to_account_info(),
                    authority: ctx.accounts.market.to_account_info(),
                },
            ),
            payout,
        )?;
    }

    

    //just for increased redundancy because the bet account should be closed after
    bet.amount = 0;
    bet.claimed = true;
    bet.initialized = false;

    Ok(())
}

#[derive(Accounts)]
pub struct ClaimBet<'info> {
    #[account(
        mut,
        seeds = [
            market.creator.key().as_ref(), 
            &hash_to_bytes(&market.feed_id),
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

    #[account(
        mut,
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

    pub price_update: Account<'info, PriceUpdateV2>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}
