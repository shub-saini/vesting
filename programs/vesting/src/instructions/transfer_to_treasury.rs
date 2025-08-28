use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::{error::CustomError, events::TokensTransferedToTreasury, state::VestingAccount};

pub fn transfer_to_treasury(ctx: Context<TransferToTreasury>, amount: u64) -> Result<()> {
    let transfer_token_cpi_accounts = TransferChecked {
        mint: ctx.accounts.mint.to_account_info(),
        from: ctx.accounts.funder_ata.to_account_info(),
        to: ctx.accounts.treasury_token_account.to_account_info(),
        authority: ctx.accounts.funder.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_context = CpiContext::new(cpi_program, transfer_token_cpi_accounts);
    let decimals = ctx.accounts.mint.decimals;
    transfer_checked(cpi_context, amount, decimals)?;

    emit!(TokensTransferedToTreasury {
        vesting_account: ctx.accounts.vesting_account.key(),
        funder: ctx.accounts.funder.key(),
        amount
    });
    Ok(())
}

#[derive(Accounts)]
pub struct TransferToTreasury<'info> {
    #[account(mut)]
    pub funder: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::token_program = token_program,
        associated_token::authority = funder
    )]
    pub funder_ata: InterfaceAccount<'info, TokenAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        constraint = vesting_account.mint == mint.key() @CustomError::InvalidMint
    )]
    pub vesting_account: Account<'info, VestingAccount>,

    #[account(
        mut,
        seeds = [b"vesting_treasury", vesting_account.key().as_ref()],
        bump = vesting_account.treasury_bump,
        constraint = treasury_token_account.mint == vesting_account.mint @CustomError::InvalidMint

    )]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}
