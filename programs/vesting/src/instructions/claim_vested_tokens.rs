use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{
    events::TokensClaimed,
    state::{BeneficiaryAccount, VestingAccount},
    CustomError,
};

pub fn handler(ctx: Context<ClaimTokens>) -> Result<()> {
    let beneficiary_vesting_account = &mut ctx.accounts.beneficiary_vesting_account;
    let now = Clock::get()?.unix_timestamp;

    require!(
        beneficiary_vesting_account.cliff_time < now,
        CustomError::ClaimNotAvailableYet
    );

    let total_vesting_time = beneficiary_vesting_account
        .end_time
        .saturating_sub(beneficiary_vesting_account.start_time);

    let effective_time = if beneficiary_vesting_account.revoke_at != None {
        beneficiary_vesting_account.revoke_at.unwrap()
    } else {
        now
    };

    let time_since_start = effective_time.saturating_sub(beneficiary_vesting_account.start_time);

    let vested_amount = if effective_time > beneficiary_vesting_account.end_time {
        beneficiary_vesting_account.total_amount
    } else {
        (beneficiary_vesting_account.total_amount * time_since_start) / total_vesting_time
    };

    let claimable_amount =
        vested_amount.saturating_sub(beneficiary_vesting_account.total_withdrawn);

    require!(claimable_amount > 0, CustomError::NothingToClaim);

    let transfer_token_cpi_account = TransferChecked {
        mint: ctx.accounts.mint.to_account_info(),
        from: ctx.accounts.treasury_token_account.to_account_info(),
        to: ctx.accounts.beneficiary_ata.to_account_info(),
        authority: ctx.accounts.treasury_token_account.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let vesting_account_key = ctx.accounts.vesting_account.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"vesting_treasury",
        vesting_account_key.as_ref(),
        &[ctx.accounts.vesting_account.treasury_bump],
    ]];

    let decimals = ctx.accounts.mint.decimals;
    let cpi_context =
        CpiContext::new_with_signer(cpi_program, transfer_token_cpi_account, signer_seeds);
    transfer_checked(cpi_context, claimable_amount as u64, decimals)?;
    beneficiary_vesting_account.total_withdrawn += claimable_amount;

    emit!(TokensClaimed {
        beneficiary: beneficiary_vesting_account.beneficiary.key(),
        vesting_account: vesting_account_key,
        timestamp: now,
        amount_claimed: claimable_amount
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(company_name: String, id: u64)]
pub struct ClaimTokens<'info> {
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = beneficiary,
        associated_token::mint = mint,
        associated_token::authority = beneficiary,
        associated_token::token_program = token_program
    )]
    pub beneficiary_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        // seeds = [b"vesting_account", company_name.as_bytes(), &id.to_le_bytes()],
        // bump = vesting_account.bump,                        
        has_one = treasury_token_account,
        has_one = mint
    )]
    pub vesting_account: Account<'info, VestingAccount>,
    #[account(mut)]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        // seeds = [b"beneficiary_vesting_account", beneficiary.key().as_ref(), vesting_account.key().as_ref()],
        // bump = beneficiary_vesting_account.bump,
        has_one = beneficiary,
        has_one = vesting_account
    )]
    pub beneficiary_vesting_account: Account<'info, BeneficiaryAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
