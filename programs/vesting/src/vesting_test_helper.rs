use std::{cell::Cell, str::FromStr};

use anchor_lang::prelude::*;
use anchor_lang::{
    prelude::borsh::BorshSerialize, system_program, AnchorDeserialize, AnchorSerialize,
};
use litesvm::LiteSVM;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_kite::{create_token_mint, deploy_program};
use solana_pubkey::Pubkey;
use solana_signer::Signer;

pub const PROGRAM_ID: &str = "FJBXuvApEoWjHNF4x4KBUMJmRuBmYyhUpcUT4etKR8iZ";
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

pub struct VestingTestEnvironment {
    pub litesvm: LiteSVM,
    pub program_id: Pubkey,
    pub employer: Keypair,
    pub token_mint: Pubkey,
    pub _worker: Pubkey,
}

pub fn setup_vesting_test() -> VestingTestEnvironment {
    let mut litesvm = LiteSVM::new();
    let program_id = get_program_id();

    deploy_program(&mut litesvm, &program_id, "../../target/deploy/vesting.so").unwrap();

    let employer = Keypair::new();
    litesvm
        .airdrop(&employer.pubkey(), 1 * LAMPORTS_PER_SOL)
        .unwrap();

    let token_mint = create_token_mint(&mut litesvm, &employer, 9).unwrap();

    let worker = Keypair::new();
    litesvm
        .airdrop(&worker.pubkey(), 1 * LAMPORTS_PER_SOL)
        .unwrap();

    VestingTestEnvironment {
        litesvm,
        program_id,
        employer,
        token_mint: token_mint.pubkey(),
        _worker: worker.pubkey(),
    }
}

pub fn get_program_id() -> Pubkey {
    Pubkey::from_str(PROGRAM_ID).unwrap()
}

thread_local! {
    static VESTING_ACCOUNT_ID_COUNTER: Cell<u64> = Cell::new(1);
}

pub fn generate_vesting_account_id() -> u64 {
    VESTING_ACCOUNT_ID_COUNTER.with(|counter| {
        let id = counter.get();
        counter.set(id + 1);
        id
    })
}

pub fn get_initialize_vesting_discriminator() -> Vec<u8> {
    let discriminator_input = b"global:initialize_vesting";
    anchor_lang::solana_program::hash::hash(discriminator_input).to_bytes()[..8].to_vec()
}

pub struct InitializeVestingAccounts {
    pub employer: Pubkey,
    pub mint: Pubkey,
    pub vesting_account: Pubkey,
    pub treasury_token_account: Pubkey,
    pub token_program: Pubkey,
    pub system_program: Pubkey,
}

pub fn build_initialize_vesting_accounts(
    employer: Pubkey,
    mint: Pubkey,
    vesting_account: Pubkey,
    treasury_token_account: Pubkey,
) -> InitializeVestingAccounts {
    let token_prog = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
    println!("Using token program: {:?}", token_prog);

    InitializeVestingAccounts {
        employer,
        mint,
        vesting_account,
        treasury_token_account,
        token_program: spl_token::ID,
        system_program: system_program::ID,
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeVestingParams {
    pub id: u64,
    pub company_name: Vec<u8>,
}

pub fn build_initialize_vesting_instruction(
    company_name: String,
    vesting_account_id: u64,
    accounts: InitializeVestingAccounts,
) -> Instruction {
    let mut instruction_data = get_initialize_vesting_discriminator();
    // instruction_data.extend_from_slice(&vesting_account_id.to_le_bytes());

    let params = InitializeVestingParams {
        id: vesting_account_id,
        company_name: company_name.into_bytes(),
    };

    params.serialize(&mut instruction_data).unwrap();

    let account_metas = vec![
        AccountMeta::new(accounts.employer, true),
        AccountMeta::new_readonly(accounts.mint, false),
        AccountMeta::new(accounts.vesting_account, false),
        AccountMeta::new(accounts.treasury_token_account, false),
        AccountMeta::new_readonly(accounts.token_program, false),
        AccountMeta::new_readonly(accounts.system_program, false),
    ];

    Instruction {
        program_id: get_program_id(),
        accounts: account_metas,
        data: instruction_data,
    }
}
