use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod errors;

use instructions::*;

declare_id!("9AhsF4FXa6GPqVWJEaCdPeK3jptuGPfZpDk24Co5odsf");

#[program]
pub mod myla_program {
    use super::*;

    /// Creates a new prediction pool for a unique (match, asset, strike_level, strike_minute) combo.
    pub fn create_pool(
        ctx: Context<CreatePool>,
        match_id: String,
        asset: String,
        strike_level: u16,
        strike_minute: u8,
        deadline: i64,
        commission_rate: u16,
    ) -> Result<()> {
        instructions::create_pool::handler(
            ctx,
            match_id,
            asset,
            strike_level,
            strike_minute,
            deadline,
            commission_rate,
        )
    }

    /// Places a bet on an existing pool. Transfers SOL from the user into the pool vault.
    pub fn place_bet(
        ctx: Context<PlaceBet>,
        side: u8,
        amount: u64,
    ) -> Result<()> {
        instructions::place_bet::handler(ctx, side, amount)
    }

    /// Oracle-only instruction. Resolves the pool with the actual stat value from TxODDS.
    pub fn resolve_pool(
        ctx: Context<ResolvePool>,
        actual_value: u16,
    ) -> Result<()> {
        instructions::resolve_pool::handler(ctx, actual_value)
    }

    /// Allows a winning bettor to claim their proportional share of the pool.
    pub fn claim_winnings(ctx: Context<ClaimWinnings>) -> Result<()> {
        instructions::claim_winnings::handler(ctx)
    }

    /// Refunds a bettor's stake if the pool is one-sided or was never resolved.
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        instructions::refund::handler(ctx)
    }
}

#[derive(Accounts)]
pub struct Initialize {}
