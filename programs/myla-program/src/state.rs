use anchor_lang::prelude::*;

/// ─── Pool Account ──────────────────────────────────────────────────────
/// PDA Seeds: ["pool", match_id, asset, strike_level.to_le_bytes(), strike_minute]
/// Each unique prediction target gets exactly one pool.
#[account]
#[derive(InitSpace)]
pub struct Pool {
    /// TxODDS fixture ID (e.g., "12345")
    #[max_len(32)]
    pub match_id: String,

    /// Asset type: "corners" | "goals" | "cards"
    #[max_len(16)]
    pub asset: String,

    /// Strike level scaled ×10 to avoid floats (e.g., 65 = 6.5)
    pub strike_level: u16,

    /// The minute at which this prediction resolves (e.g., 45)
    pub strike_minute: u8,

    /// Unix timestamp after which no more bets are allowed
    pub deadline: i64,

    /// Total lamports staked on the Over side
    pub over_total: u64,

    /// Total lamports staked on the Under side
    pub under_total: u64,

    /// Number of Over bettors
    pub over_count: u32,

    /// Number of Under bettors
    pub under_count: u32,

    /// Whether the oracle has resolved this pool
    pub resolved: bool,

    /// 0 = Over won, 1 = Under won. None if unresolved.
    pub winning_side: Option<u8>,

    /// The actual stat value at the strike minute (scaled ×10)
    pub actual_value: Option<u16>,

    /// Commission rate in basis points (e.g., 500 = 5.00%)
    pub commission_rate: u16,

    /// MYLA treasury wallet that receives the commission
    pub commission_wallet: Pubkey,

    /// The authorized oracle keypair that can resolve this pool
    pub oracle: Pubkey,

    /// PDA bump seed
    pub bump: u8,
}

/// ─── Bet Account ───────────────────────────────────────────────────────
/// PDA Seeds: ["bet", pool_pubkey, user_pubkey]
/// One per user per pool. Tracks their individual stake and chosen side.
#[account]
#[derive(InitSpace)]
pub struct Bet {
    /// The pool this bet belongs to
    pub pool: Pubkey,

    /// The bettor's wallet public key
    pub user: Pubkey,

    /// 0 = Over, 1 = Under
    pub side: u8,

    /// Lamports staked by this user
    pub amount: u64,

    /// Whether the user has already claimed their winnings
    pub claimed: bool,

    /// PDA bump seed
    pub bump: u8,
}
