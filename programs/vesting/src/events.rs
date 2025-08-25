use anchor_lang::prelude::*;

#[event]
pub struct VestingAccountCreated {
    pub id: u64,
    pub company_name: String,
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub treasury: Pubkey,
}

#[event]
pub struct VestingScheduleInitialized {
    pub beneficiary: Pubkey,
    pub vesting_account: Pubkey,
    pub start_time: i64,
    pub end_time: i64,
    pub total_amount: u64,
    pub cliff_time: i64,
}
