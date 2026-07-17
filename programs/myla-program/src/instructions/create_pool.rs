use anchor_lang::prelude::*;
use crate::state::Pool;
use crate::errors::MylaError;

/// Accounts required to create a new prediction pool.
#[derive(Accounts)]
#[instruction(
    match_id: String,
    asset: String,
    strike_level: u16,
    strike_minute: u8,
)]
pub struct CreatePool<'info> {
    /// The user who initiates the pool creation (pays for rent).
    #[account(mut)]
    pub creator: Signer<'info>,

    /// The Pool PDA account to be initialized.
    /// Seeds: ["pool", match_id, asset, strike_level bytes, strike_minute]
    #[account(
        init,
        payer = creator,
        space = 8 + Pool::INIT_SPACE,
        seeds = [
            b"pool",
            match_id.as_bytes(),
            asset.as_bytes(),
            &strike_level.to_le_bytes(),
            &[strike_minute],
        ],
        bump,
    )]
    pub pool: Account<'info, Pool>,

    /// The oracle keypair authorized to resolve this pool.
    /// CHECK: This is just stored as the authorized resolver pubkey.
    pub oracle: UncheckedAccount<'info>,

    /// The commission wallet that receives protocol fees.
    /// CHECK: This is just stored as the commission recipient pubkey.
    pub commission_wallet: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreatePool>,
    match_id: String,
    asset: String,
    strike_level: u16,
    strike_minute: u8,
    deadline: i64,
    commission_rate: u16,
) -> Result<()> {
    // Validate inputs
    require!(match_id.len() <= 32, MylaError::MatchIdTooLong);
    require!(asset.len() <= 16, MylaError::AssetTooLong);
    require!(commission_rate <= 10_000, MylaError::InvalidCommissionRate);

    let clock = Clock::get()?;
    require!(deadline > clock.unix_timestamp, MylaError::DeadlineInPast);

    let pool = &mut ctx.accounts.pool;

    pool.match_id = match_id;
    pool.asset = asset;
    pool.strike_level = strike_level;
    pool.strike_minute = strike_minute;
    pool.deadline = deadline;
    pool.over_total = 0;
    pool.under_total = 0;
    pool.over_count = 0;
    pool.under_count = 0;
    pool.resolved = false;
    pool.winning_side = None;
    pool.actual_value = None;
    pool.commission_rate = commission_rate;
    pool.commission_wallet = ctx.accounts.commission_wallet.key();
    pool.oracle = ctx.accounts.oracle.key();
    pool.bump = ctx.bumps.pool;

    msg!(
        "Pool created: {} {} strike={} min={}",
        pool.match_id,
        pool.asset,
        pool.strike_level,
        pool.strike_minute,
    );

    Ok(())
}
