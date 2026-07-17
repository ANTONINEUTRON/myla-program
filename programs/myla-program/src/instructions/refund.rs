use anchor_lang::prelude::*;
use crate::state::{Pool, Bet};
use crate::errors::MylaError;

/// Grace period after deadline before refunds become available (24 hours in seconds).
const REFUND_GRACE_PERIOD: i64 = 86_400;

/// Accounts required to refund a bettor's stake.
#[derive(Accounts)]
pub struct Refund<'info> {
    /// The user requesting a refund.
    #[account(mut)]
    pub user: Signer<'info>,

    /// The Pool PDA.
    #[account(mut)]
    pub pool: Account<'info, Pool>,

    /// The user's Bet PDA on this pool.
    #[account(
        mut,
        seeds = [b"bet", pool.key().as_ref(), user.key().as_ref()],
        bump = bet.bump,
        constraint = bet.user == user.key(),
        constraint = bet.pool == pool.key(),
        close = user,
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

pub fn handler(ctx: Context<Refund>) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let bet = &ctx.accounts.bet;

    require!(!bet.claimed, MylaError::AlreadyClaimed);

    let clock = Clock::get()?;

    // Refund is allowed in two cases:
    // 1. Pool is one-sided (only Over or only Under has bets) and deadline has passed.
    // 2. Pool was not resolved within the grace period after deadline.
    let is_one_sided = pool.over_total == 0 || pool.under_total == 0;
    let deadline_passed = clock.unix_timestamp >= pool.deadline;
    let grace_expired = clock.unix_timestamp >= pool.deadline + REFUND_GRACE_PERIOD;

    let eligible = if pool.resolved {
        // If resolved, no refund — use claim_winnings instead
        false
    } else if is_one_sided && deadline_passed {
        // One-sided pool after deadline → everyone gets refunded
        true
    } else if grace_expired {
        // Oracle didn't resolve within 24h → refund everyone
        true
    } else {
        false
    };

    require!(eligible, MylaError::RefundNotEligible);

    let refund_amount = bet.amount;

    // Transfer refund from vault to user
    **ctx.accounts.vault.to_account_info().try_borrow_mut_lamports()? -= refund_amount;
    **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += refund_amount;

    // Update pool totals
    let pool = &mut ctx.accounts.pool;
    if bet.side == 0 {
        pool.over_total = pool.over_total.saturating_sub(refund_amount);
        pool.over_count = pool.over_count.saturating_sub(1);
    } else {
        pool.under_total = pool.under_total.saturating_sub(refund_amount);
        pool.under_count = pool.under_count.saturating_sub(1);
    }

    msg!(
        "Refund issued: user={} amount={} lamports",
        bet.user,
        refund_amount,
    );

    // The bet account is closed via the `close = user` constraint,
    // returning rent back to the user automatically.

    Ok(())
}
