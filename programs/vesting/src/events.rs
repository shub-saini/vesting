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
    pub total_amount: i64,
    pub cliff_time: i64,
}

#[event]
pub struct TokensClaimed {
    pub vesting_account: Pubkey,
    pub beneficiary: Pubkey,
    pub timestamp: i64,
    pub amount_claimed: i64,
}

#[event]
pub struct TokensTransferedToTreasury {
    pub vesting_account: Pubkey,
    pub funder: Pubkey,
    pub amount: u64,
}

#[event]
pub struct AdminChanged {
    pub vesting_account: Pubkey,
    pub old_admin: Pubkey,
    pub new_admin: Pubkey,
}

#[event]
pub struct BeneficiaryAccountRevoked {
    pub vestng_account: Pubkey,
    pub beneficiary: Pubkey,
}
