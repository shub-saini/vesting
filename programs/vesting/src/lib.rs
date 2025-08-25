#![allow(unexpected_cfgs)]

pub mod constant;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

use constant::*;
use error::*;
use events::*;
use instructions::*;
use state::*;

declare_id!("FJBXuvApEoWjHNF4x4KBUMJmRuBmYyhUpcUT4etKR8iZ");

#[program]
pub mod vesting {
    use super::*;

    pub fn create_vesting_account(
        ctx: Context<CreateVestingAccount>,
        id: u64,
        company_name: String,
    ) -> Result<()> {
        instructions::create_vesting_account::handler(ctx, id, company_name)
    }

    pub fn initialize_vesting_schedule(
        ctx: Context<InitializeVestingSchedule>,
        start_time: i64,
        end_time: i64,
        total_amount: u64,
        cliff_time: i64,
    ) -> Result<()> {
        instructions::initialize_vesting_schedule::handler(
            ctx,
            start_time,
            end_time,
            total_amount,
            cliff_time,
        )
    }

    // pub fn claim_vested_tokens() -> Result<()> {}

    // pub fn change_admin() -> Result<()> {}

    // pub fn update_vesting_schedule() -> Result<()> {}

    // pub fn create_batch_vesting_schedules() -> Result<()> {}

    // pub fn transfer_tokens_to_treasury() -> Result<()> {}

    // pub fn pause_vesting() -> Result<()> {}

    // pub fn resume_vesting() -> Result<()> {}

    // schedule for multiple beneficiaries ????
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod vesting_test_helper;
