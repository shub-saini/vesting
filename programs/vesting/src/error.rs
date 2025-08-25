use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Company name should not be greater then 50 characters")]
    CompanyNameTooLong,
    #[msg("Time constraints not satisfied")]
    InvalidVestingSchedule,
    #[msg("Total claimable vesting amount cannot be zero")]
    VestingAmountCannotBeZero,
}
