use anchor_lang::prelude::*;

#[error_code]
pub enum MylaError {
    #[msg("The deadline must be in the future")]
    DeadlineInPast,

    #[msg("The commission rate must be between 0 and 10000 basis points (100%)")]
    InvalidCommissionRate,

    #[msg("The pool has already been resolved")]
    PoolAlreadyResolved,

    #[msg("The betting deadline has passed")]
    DeadlinePassed,

    #[msg("The minimum stake is 0.01 SOL (10,000,000 lamports)")]
    StakeTooSmall,

    #[msg("Invalid side: must be 0 (Over) or 1 (Under)")]
    InvalidSide,

    #[msg("Only the authorized oracle can resolve this pool")]
    UnauthorizedOracle,

    #[msg("The pool deadline has not passed yet")]
    DeadlineNotReached,

    #[msg("The pool has not been resolved yet")]
    PoolNotResolved,

    #[msg("Your bet is on the losing side")]
    NotAWinner,

    #[msg("You have already claimed your winnings")]
    AlreadyClaimed,

    #[msg("The pool is not eligible for a refund")]
    RefundNotEligible,

    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,

    #[msg("Match ID is too long (max 32 characters)")]
    MatchIdTooLong,

    #[msg("Asset name is too long (max 16 characters)")]
    AssetTooLong,
}
