use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{
    constant::ANCHOR_DISCRIMINATOR_SIZE,
    events::VestingScheduleInitialized,
    state::{BeneficiaryAccount, VestingAccount},
    CustomError,
};

pub fn handler(
    ctx: Context<InitializeVestingSchedule>,
    start_time: i64,
    end_time: i64,
    total_amount: u64,
    cliff_time: i64,
) -> Result<()> {
    require!(
        start_time < end_time && start_time < cliff_time && cliff_time < end_time,
        CustomError::InvalidVestingSchedule
    );
    require!(total_amount != 0, CustomError::VestingAmountCannotBeZero);

    ctx.accounts
        .beneficiary_vesting_account
        .set_inner(BeneficiaryAccount {
            beneficiary: ctx.accounts.beneficiary.key(),
            vesting_account: ctx.accounts.vesting_account.key(),
            start_time,
            end_time,
            total_amount,
            total_withdrawn: 0,
            cliff_time,
            bump: ctx.bumps.beneficiary_vesting_account,
        });

    emit!(VestingScheduleInitialized {
        beneficiary: ctx.accounts.beneficiary.key(),
        vesting_account: ctx.accounts.vesting_account.key(),
        start_time,
        end_time,
        total_amount,
        cliff_time
    });
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeVestingSchedule<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    pub vesting_account: Account<'info, VestingAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    pub beneficiary: SystemAccount<'info>,
    #[account(
        init,
        payer = admin,
        space = ANCHOR_DISCRIMINATOR_SIZE + BeneficiaryAccount::INIT_SPACE,
        seeds = [b"beneficiary_vesting_schedule", beneficiary.key().as_ref(), vesting_account.key().as_ref()],
        bump
    )]
    pub beneficiary_vesting_account: Account<'info, BeneficiaryAccount>,
    pub system_program: Program<'info, System>,
}
