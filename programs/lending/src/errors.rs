use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient Funds")]
    InsufficientFunds,
    #[msg("Over the Borrowable Amount")]
    OverTheBorrowableAmount,
    #[msg("Insufficient Repay Amount")]
    InsufficientRepayAmount,  
    #[msg("Health Factor is above the threshold")]
    HealthFactorAboveThreshold,
}