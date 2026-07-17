use anchor_lang::prelude::*;
use crate::state::Pool;
use crate::errors::MylaError;

/// Accounts required for the oracle to resolve a pool.
#[derive(Accounts)]
pub struct ResolvePool<'info> {
    /// The oracle signer — must match `pool.oracle`.
    #[account(mut)]
    pub oracle: Signer<'info>,

    /// The Pool PDA to resolve.
    #[account(
        mut,
        constraint = pool.oracle == oracle.key() @ MylaError::UnauthorizedOracle,
    )]
    pub pool: Account<'info, Pool>,

    /// The pool vault PDA holding escrowed SOL.
    /// CHECK: This is a PDA used as a SOL vault.
    #[account(
        mut,
        seeds = [b"vault", pool.key().as_ref()],
        bump,
    )]
    pub vault: UncheckedAccount<'info>,

    /// The commission wallet that receives protocol fees.
    /// CHECK: Must match pool.commission_wallet.
    #[account(
        mut,
        constraint = commission_wallet.key() == pool.commission_wallet,
    )]
    pub commission_wallet: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ResolvePool>, actual_value: u16) -> Result<()> {
    let pool = &ctx.accounts.pool;

    require!(!pool.resolved, MylaError::PoolAlreadyResolved);

    let clock = Clock::get()?;
    require!(clock.unix_timestamp >= pool.deadline, MylaError::DeadlineNotReached);

    // Determine the winning side
    // actual_value and strike_level are both scaled ×10
    // Over wins if actual_value > strike_level, Under wins if actual_value <= strike_level
    let winning_side: u8 = if actual_value > pool.strike_level { 0 } else { 1 };

    let total_pool = pool.over_total
        .checked_add(pool.under_total)
        .ok_or(MylaError::ArithmeticOverflow)?;

    // Calculate and transfer commission from vault to commission wallet
    let commission = total_pool
        .checked_mul(pool.commission_rate as u64)
        .ok_or(MylaError::ArithmeticOverflow)?
        .checked_div(10_000)
        .ok_or(MylaError::ArithmeticOverflow)?;

    if commission > 0 {
        // Transfer commission from vault PDA to commission wallet
        let pool_key = ctx.accounts.pool.key();
        let vault_bump = ctx.bumps.vault;
        let vault_seeds: &[&[u8]] = &[b"vault", pool_key.as_ref(), &[vault_bump]];

        **ctx.accounts.vault.to_account_info().try_borrow_mut_lamports()? -= commission;
        **ctx.accounts.commission_wallet.to_account_info().try_borrow_mut_lamports()? += commission;

        msg!("Commission transferred: {} lamports", commission);

        // Silence unused variable warning — seeds kept for potential future CPI use
        let _ = vault_seeds;
    }

    // Update pool state
    let pool = &mut ctx.accounts.pool;
    pool.resolved = true;
    pool.winning_side = Some(winning_side);
    pool.actual_value = Some(actual_value);

    msg!(
        "Pool resolved: actual_value={} strike_level={} winner={}",
        actual_value,
        pool.strike_level,
        if winning_side == 0 { "Over" } else { "Under" },
    );

    Ok(())
}
