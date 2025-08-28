use anchor_lang::prelude::*;

use crate::{error::CustomError, events::AdminChanged, state::VestingAccount};

pub fn handler(ctx: Context<ChangeAdmin>) -> Result<()> {
    let vesting_account = &mut ctx.accounts.vesting_account;
    let old_admin = ctx.accounts.admin.key();

    vesting_account.admin = ctx.accounts.new_admin.key();

    emit!(AdminChanged {
        vesting_account: vesting_account.key(),
        old_admin,
        new_admin: ctx.accounts.new_admin.key()
    });
    Ok(())
}

#[derive(Accounts)]
pub struct ChangeAdmin<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        constraint = vesting_account.admin == admin.key() @CustomError::UnAuthorized
    )]
    pub vesting_account: Account<'info, VestingAccount>,
    pub new_admin: SystemAccount<'info>,
}
