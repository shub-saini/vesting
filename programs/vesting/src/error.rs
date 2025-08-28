use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Company name should not be greater then 50 characters")]
    CompanyNameTooLong,
    #[msg("Time constraints not satisfied")]
    InvalidVestingSchedule,
    #[msg("Total claimable vesting amount should be positive")]
    VestingAmountShoulBePositive,
    #[msg("Vesting has not started yet")]
    ClaimNotAvailableYet,
    #[msg("There is nothing to claim")]
    NothingToClaim,
    #[msg("Invalid Mint for vesting account")]
    InvalidMint,
    #[msg("This operation can only be perfomed by the admin")]
    UnAuthorized,
}
