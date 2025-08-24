use anchor_lang::AccountDeserialize;
// use anchor_lang::Key;
use solana_kite::{get_pda_and_bump, seeds, send_transaction_from_instructions};
use solana_message::Message;
use solana_signer::Signer;
use solana_transaction::Transaction;

use crate::{
    state::VestingAccount,
    vesting_test_helper::{
        build_initialize_vesting_accounts, build_initialize_vesting_instruction,
        generate_vesting_account_id, setup_vesting_test,
    },
};

#[test]
pub fn test_initialize_vesting() {
    let mut test_environment = setup_vesting_test();

    let company_name = String::from("company");

    let vesting_id = generate_vesting_account_id();

    let (vesting_account, _vesting_bump) = get_pda_and_bump(
        &seeds![
            b"vesting_account".as_ref(),
            test_environment.employer.pubkey().as_ref(),
            company_name.as_bytes(),
            vesting_id.to_le_bytes().as_ref(),
        ],
        &test_environment.program_id,
    );

    let (treasury_token_account, _treasury_bump) = get_pda_and_bump(
        &seeds![b"vesting_treasury".as_ref(), vesting_account.as_ref()],
        &test_environment.program_id,
    );

    let initializing_vesting_accounts = build_initialize_vesting_accounts(
        test_environment.employer.pubkey(),
        test_environment.token_mint,
        vesting_account,
        treasury_token_account,
    );

    let initialize_vesting_instruction = build_initialize_vesting_instruction(
        company_name.clone(),
        vesting_id,
        initializing_vesting_accounts,
    );

    let blockhash = test_environment.litesvm.latest_blockhash();
    let message = Message::new(
        &[initialize_vesting_instruction],
        Some(&test_environment.employer.pubkey()),
    );

    let tx = Transaction::new(&[&test_environment.employer], message, blockhash);

    let tx_res = test_environment.litesvm.send_transaction(tx).unwrap();

    println!("Logs: {}", tx_res.pretty_logs());

    // let result = send_transaction_from_instructions(
    //     &mut test_environment.litesvm,
    //     vec![initialize_vesting_instruction],
    //     &[&test_environment.employer],
    //     &test_environment.employer.pubkey(),
    // );
    // assert!(result.is_ok(), "Valid account to be created");

    let vesting_account_data = test_environment
        .litesvm
        .get_account(&vesting_account)
        .expect("Vesting Account should Exists")
        .data;

    let vesting_data = VestingAccount::try_deserialize(&mut vesting_account_data.as_slice())
        .expect("Should be able to deserialize vesting account");

    assert_eq!(vesting_data.company_name, company_name);
    assert_eq!(vesting_data.id, vesting_id);
}
