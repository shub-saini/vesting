use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{VestingAccount, ANCHOR_DISCRIMINATOR_SIZE};

pub fn handler(ctx: Context<CreateVestingAccount>, id: u64, company_name: String) -> Result<()> {
    let vesting_account = &mut ctx.accounts.vesting_account;

    vesting_account.set_inner(VestingAccount {
        id,
        employer: ctx.accounts.employer.key(),
        mint: ctx.accounts.mint.key(),
        treasury_token_account: ctx.accounts.treasury_token_account.key(),
        company_name: company_name,
        treasury_bump: ctx.bumps.treasury_token_account,
        bump: ctx.bumps.vesting_account,
    });
    Ok(())
}

#[derive(Accounts)]
#[instruction(id: u64, company_name: String)]
pub struct CreateVestingAccount<'info> {
    #[account(mut)]
    pub employer: Signer<'info>,
    #[account(mint::token_program = token_program)]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = employer,
        space = ANCHOR_DISCRIMINATOR_SIZE + VestingAccount::INIT_SPACE,
        seeds = [b"vesting_account", employer.key().as_ref(), company_name.as_bytes(), &id.to_le_bytes()],
        // seeds = [b"vesting_account".as_ref(), company_name.as_bytes(), id.to_le_bytes().as_ref()],
        bump
    )]
    pub vesting_account: Account<'info, VestingAccount>,
    #[account(
        init,
        payer = employer,
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
