use anchor_lang::prelude::*;
use crate::state::{Pool, Bet};
use crate::errors::MylaError;

/// Accounts required for a winning bettor to claim their payout.
#[derive(Accounts)]
pub struct ClaimWinnings<'info> {
    /// The user claiming their winnings.
    #[account(mut)]
    pub user: Signer<'info>,

    /// The resolved Pool PDA.
    #[account(
        constraint = pool.resolved @ MylaError::PoolNotResolved,
    )]
    pub pool: Account<'info, Pool>,

    /// The user's Bet PDA on this pool.
    #[account(
        mut,
        seeds = [b"bet", pool.key().as_ref(), user.key().as_ref()],
        bump = bet.bump,
        constraint = bet.user == user.key(),
        constraint = bet.pool == pool.key(),
    )]
    pub bet: Account<'info, Bet>,

    /// The pool vault PDA holding escrowed SOL.
    /// CHECK: This is a PDA used as a SOL vault.
    #[account(
        mut,
        seeds = [b"vault", pool.key().as_ref()],
        bump,
    )]
    pub vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ClaimWinnings>) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let bet = &ctx.accounts.bet;

    // Ensure the bet is on the winning side
    let winning_side = pool.winning_side.ok_or(MylaError::PoolNotResolved)?;
    require!(bet.side == winning_side, MylaError::NotAWinner);
    require!(!bet.claimed, MylaError::AlreadyClaimed);

    // Calculate payout:
    // payout = (user_stake / winning_side_total) × (total_pool - commission)
    let total_pool = pool.over_total
        .checked_add(pool.under_total)
        .ok_or(MylaError::ArithmeticOverflow)?;

    let commission = total_pool
        .checked_mul(pool.commission_rate as u64)
        .ok_or(MylaError::ArithmeticOverflow)?
        .checked_div(10_000)
        .ok_or(MylaError::ArithmeticOverflow)?;

    let distributable = total_pool
        .checked_sub(commission)
        .ok_or(MylaError::ArithmeticOverflow)?;

    let winning_side_total = if winning_side == 0 {
        pool.over_total
    } else {
        pool.under_total
    };

    // payout = (bet.amount × distributable) / winning_side_total
    // Use u128 intermediate to avoid overflow on multiplication
    let payout = (bet.amount as u128)
        .checked_mul(distributable as u128)
        .ok_or(MylaError::ArithmeticOverflow)?
        .checked_div(winning_side_total as u128)
        .ok_or(MylaError::ArithmeticOverflow)? as u64;

    // Transfer payout from vault to user
    **ctx.accounts.vault.to_account_info().try_borrow_mut_lamports()? -= payout;
    **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += payout;

    // Mark as claimed
    let bet = &mut ctx.accounts.bet;
    bet.claimed = true;

    msg!(
        "Winnings claimed: user={} payout={} lamports",
        bet.user,
        payout,
    );

    Ok(())
}
