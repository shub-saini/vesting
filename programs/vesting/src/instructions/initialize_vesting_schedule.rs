use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};

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
    total_amount: i64,
    cliff_time: i64,
) -> Result<()> {
    require!(
        start_time < end_time && start_time < cliff_time && cliff_time < end_time,
        CustomError::InvalidVestingSchedule
    );
    require!(total_amount > 0, CustomError::VestingAmountShoulBePositive);

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
            revoke_at: None,
            bump: ctx.bumps.beneficiary_vesting_account,
        });

    let treasury_amount = ctx.accounts.treasury_token_account.amount;

    ctx.accounts.vesting_account.total_token_obligation += total_amount as u64;

    require!(
        treasury_amount > ctx.accounts.vesting_account.total_token_obligation,
        CustomError::NotEnoughTokensInTreasury
    );

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
    #[account(
        mut,
        constraint = vesting_account.admin == admin.key() @CustomError::UnAuthorized,
        constraint = vesting_account.mint == mint.key() @CustomError::InvalidMint
    )]
    pub vesting_account: Account<'info, VestingAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    pub beneficiary: SystemAccount<'info>,
    #[account(
        init,
        payer = admin,
        space = ANCHOR_DISCRIMINATOR_SIZE + BeneficiaryAccount::INIT_SPACE,
        seeds = [b"beneficiary_vesting_account", beneficiary.key().as_ref(), vesting_account.key().as_ref()],
        bump
    )]
    pub beneficiary_vesting_account: Account<'info, BeneficiaryAccount>,
    #[account(
        seeds = [b"vesting_treasury", vesting_account.key().as_ref()],
        bump
    )]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
}
