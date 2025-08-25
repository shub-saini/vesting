use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct BeneficiaryAccount {
    pub beneficiary: Pubkey,
    pub vesting_account: Pubkey,
    pub start_time: i64,
    pub end_time: i64,
    pub total_amount: u64,
    pub total_withdrawn: u64,
    pub cliff_time: i64,
    pub bump: u8,
}
