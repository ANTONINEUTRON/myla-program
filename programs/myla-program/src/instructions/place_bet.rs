use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::state::{Pool, Bet};
use crate::errors::MylaError;

/// Minimum stake: 0.01 SOL = 10,000,000 lamports
const MIN_STAKE_LAMPORTS: u64 = 10_000_000;

/// Accounts required to place a bet on an existing pool.
#[derive(Accounts)]
pub struct PlaceBet<'info> {
    /// The user placing the bet.
    #[account(mut)]
    pub user: Signer<'info>,

    /// The Pool PDA this bet belongs to.
    #[account(mut)]
    pub pool: Account<'info, Pool>,

    /// The Bet PDA for this user on this pool.
    /// Seeds: ["bet", pool_pubkey, user_pubkey]
    #[account(
        init,
        payer = user,
        space = 8 + Bet::INIT_SPACE,
        seeds = [
            b"bet",
            pool.key().as_ref(),
            user.key().as_ref(),
        ],
        bump,
    )]
    pub bet: Account<'info, Bet>,

    /// The pool vault PDA that holds escrowed SOL.
    /// Seeds: ["vault", pool_pubkey]
    /// CHECK: This is a PDA used as a SOL vault (not a token account).
    #[account(
        mut,
        seeds = [b"vault", pool.key().as_ref()],
        bump,
    )]
    pub vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<PlaceBet>, side: u8, amount: u64) -> Result<()> {
    // Validate inputs
    require!(side == 0 || side == 1, MylaError::InvalidSide);
    require!(amount >= MIN_STAKE_LAMPORTS, MylaError::StakeTooSmall);

    let pool = &ctx.accounts.pool;
    require!(!pool.resolved, MylaError::PoolAlreadyResolved);

    let clock = Clock::get()?;
    require!(clock.unix_timestamp < pool.deadline, MylaError::DeadlinePassed);

    // Transfer SOL from user to vault
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        ),
        amount,
    )?;

    // Update pool totals
    let pool = &mut ctx.accounts.pool;
    if side == 0 {
        pool.over_total = pool.over_total.checked_add(amount)
            .ok_or(MylaError::ArithmeticOverflow)?;
        pool.over_count = pool.over_count.checked_add(1)
            .ok_or(MylaError::ArithmeticOverflow)?;
    } else {
        pool.under_total = pool.under_total.checked_add(amount)
            .ok_or(MylaError::ArithmeticOverflow)?;
        pool.under_count = pool.under_count.checked_add(1)
            .ok_or(MylaError::ArithmeticOverflow)?;
    }

    // Initialize the bet account
    let bet = &mut ctx.accounts.bet;
    bet.pool = ctx.accounts.pool.key();
    bet.user = ctx.accounts.user.key();
    bet.side = side;
    bet.amount = amount;
    bet.claimed = false;
    bet.bump = ctx.bumps.bet;

    msg!(
        "Bet placed: user={} side={} amount={} lamports",
        bet.user,
        if side == 0 { "Over" } else { "Under" },
        amount,
    );

    Ok(())
}
