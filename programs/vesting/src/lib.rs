#![allow(unexpected_cfgs)]

pub mod constant;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

use constant::*;
use instructions::*;
use state::*;

declare_id!("FJBXuvApEoWjHNF4x4KBUMJmRuBmYyhUpcUT4etKR8iZ");

#[program]
pub mod vesting {
    use super::*;

    pub fn initialize_vesting(
        ctx: Context<CreateVestingAccount>,
        id: u64,
        company_name: String,
    ) -> Result<()> {
        msg!("company-name-as-bytes {:?}", company_name.as_bytes());

        instructions::initialize_vesting::handler(ctx, id, company_name)
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod vesting_test_helper;
