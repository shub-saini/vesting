use anchor_lang::prelude::*;

use crate::{
    state::{BeneficiaryAccount, VestingAccount},
    CustomError,
};

pub fn handler(ctx: Context<RevokeAccount>) -> Result<()> {
    let beneficiary_vesting_account = &mut ctx.accounts.beneficiary_vesting_account;
    require!(
        beneficiary_vesting_account.total_amount == beneficiary_vesting_account.total_withdrawn,
        CustomError::RevokeNotNeeded
    );

    let now = Clock::get()?.unix_timestamp;
    beneficiary_vesting_account.revoke_at = Some(now);

    Ok(())
}

#[derive(Accounts)]
pub struct RevokeAccount<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    pub beneficiary: SystemAccount<'info>,
    pub vesting_account: Account<'info, VestingAccount>,
    #[account(
        mut,
        seeds = [b"beneficiary_vesting_account", beneficiary.key().as_ref(), vesting_account.key().as_ref()],
        bump = beneficiary_vesting_account.bump
    )]
    pub beneficiary_vesting_account: Account<'info, BeneficiaryAccount>,
}
