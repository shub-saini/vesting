use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    events::VestingAccountCreated, CustomError, VestingAccount, ANCHOR_DISCRIMINATOR_SIZE,
};

pub fn handler(ctx: Context<CreateVestingAccount>, id: u64, company_name: String) -> Result<()> {
    require!(company_name.len() <= 50, CustomError::CompanyNameTooLong);

    ctx.accounts.vesting_account.set_inner(VestingAccount {
        id,
        admin: ctx.accounts.admin.key(),
        mint: ctx.accounts.mint.key(),
        treasury_token_account: ctx.accounts.treasury_token_account.key(),
        company_name: company_name.clone(),
        total_token_obligation: 0,
        treasury_bump: ctx.bumps.treasury_token_account,
        bump: ctx.bumps.vesting_account,
    });

    emit!(VestingAccountCreated {
        id,
        company_name,
        admin: ctx.accounts.admin.key(),
        mint: ctx.accounts.mint.key(),
        treasury: ctx.accounts.treasury_token_account.key()
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(id: u64, company_name: String)]
pub struct CreateVestingAccount<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mint::token_program = token_program)]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = admin,
        space = ANCHOR_DISCRIMINATOR_SIZE + VestingAccount::INIT_SPACE,
        seeds = [b"vesting_account", company_name.as_bytes(), &id.to_le_bytes()],
        bump
    )]
    pub vesting_account: Account<'info, VestingAccount>,
    #[account(
        init,
        payer = admin,
        token::mint = mint,
        token::authority = treasury_token_account,
        token::token_program = token_program,
        seeds = [b"vesting_treasury", vesting_account.key().as_ref()],
        bump
    )]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}
