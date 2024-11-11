mod helpers;

use helpers::{
    create_stake_deposit_authority, program_test_context_with_stake_pool_state, StakePoolAccounts,
};
use jito_bytemuck::AccountDeserialize;
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    account::AccountSharedData,
    borsh1::try_from_slice_unchecked,
    instruction::{AccountMeta, Instruction, InstructionError},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::{Transaction, TransactionError},
    transport::TransportError,
};
use stake_deposit_interceptor::{
    error::StakeDepositInterceptorError, instruction::derive_stake_pool_deposit_stake_authority,
    state::StakePoolDepositStakeAuthority,
};

#[tokio::test]
async fn test_update_deposit_stake_authority() {
    let (mut ctx, stake_pool_accounts) = program_test_context_with_stake_pool_state().await;
    let stake_pool_account = ctx
        .banks_client
        .get_account(stake_pool_accounts.stake_pool)
        .await
        .unwrap()
        .unwrap();
    let stake_pool =
        try_from_slice_unchecked::<spl_stake_pool::state::StakePool>(&stake_pool_account.data)
            .unwrap();

    let authority = Keypair::new();
    create_stake_deposit_authority(
        &mut ctx,
        &stake_pool_accounts.stake_pool,
        &stake_pool.pool_mint,
        &authority,
        None,
    )
    .await;

    let fee_wallet = Keypair::new();
    let new_authority = Keypair::new();
    let cool_down_period = 78;
    let initial_fee_rate = 20;

    let update_ix =
        stake_deposit_interceptor::instruction::create_update_deposit_stake_authority_instruction(
            &stake_deposit_interceptor::id(),
            &stake_pool_accounts.stake_pool,
            &authority.pubkey(),
            Some(new_authority.pubkey()),
            Some(fee_wallet.pubkey()),
            Some(cool_down_period),
            Some(initial_fee_rate),
        );

    let tx = Transaction::new_signed_with_payer(
        &[update_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority, &new_authority],
        ctx.last_blockhash,
    );

    ctx.banks_client.process_transaction(tx).await.unwrap();

    let (deposit_stake_authority_pubkey, _bump_seed) = derive_stake_pool_deposit_stake_authority(
        &stake_deposit_interceptor::ID,
        &stake_pool_accounts.stake_pool,
    );

    let account = ctx
        .banks_client
        .get_account(deposit_stake_authority_pubkey)
        .await
        .unwrap()
        .unwrap();

    let deposit_stake_authority =
        StakePoolDepositStakeAuthority::try_from_slice_unchecked(&account.data.as_slice()).unwrap();

    let actual_cool_down_period: u64 = deposit_stake_authority.cool_down_period.into();
    let actual_initial_fee_rate: u32 = deposit_stake_authority.inital_fee_rate.into();
    assert_eq!(actual_cool_down_period, cool_down_period);
    assert_eq!(actual_initial_fee_rate, initial_fee_rate);
    assert_eq!(deposit_stake_authority.fee_wallet, fee_wallet.pubkey());
    assert_eq!(deposit_stake_authority.authority, new_authority.pubkey());
}

async fn setup_with_ix() -> (
    ProgramTestContext,
    StakePoolAccounts,
    Keypair,
    Keypair,
    Instruction,
) {
    let (mut ctx, stake_pool_accounts) = program_test_context_with_stake_pool_state().await;
    let stake_pool_account = ctx
        .banks_client
        .get_account(stake_pool_accounts.stake_pool)
        .await
        .unwrap()
        .unwrap();
    let stake_pool =
        try_from_slice_unchecked::<spl_stake_pool::state::StakePool>(&stake_pool_account.data)
            .unwrap();

    let authority = Keypair::new();
    create_stake_deposit_authority(
        &mut ctx,
        &stake_pool_accounts.stake_pool,
        &stake_pool.pool_mint,
        &authority,
        None,
    )
    .await;

    let fee_wallet = Keypair::new();
    let new_authority = Keypair::new();
    let cool_down_period = 78;
    let initial_fee_rate = 20;

    let update_ix =
        stake_deposit_interceptor::instruction::create_update_deposit_stake_authority_instruction(
            &stake_deposit_interceptor::id(),
            &stake_pool_accounts.stake_pool,
            &authority.pubkey(),
            Some(new_authority.pubkey()),
            Some(fee_wallet.pubkey()),
            Some(cool_down_period),
            Some(initial_fee_rate),
        );
    (
        ctx,
        stake_pool_accounts,
        authority,
        new_authority,
        update_ix,
    )
}

#[tokio::test]
async fn test_fail_program_does_not_own_pda_account() {
    let (mut ctx, _stake_pool_accounts, authority, new_authority, mut init_ix) =
        setup_with_ix().await;
    init_ix.accounts[0] = AccountMeta::new(Pubkey::new_unique(), false);

    let tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority, &new_authority],
        ctx.last_blockhash,
    );

    let transaction_error: TransportError = ctx
        .banks_client
        .process_transaction(tx)
        .await
        .err()
        .expect("Should have errored")
        .into();

    match transaction_error {
        TransportError::TransactionError(TransactionError::InstructionError(_, error)) => {
            assert_eq!(error, InstructionError::IncorrectProgramId);
        }
        _ => panic!("Wrong error"),
    };
}

#[tokio::test]
async fn test_fail_authority_not_signer() {
    let (mut ctx, _stake_pool_accounts, authority, new_authority, mut init_ix) =
        setup_with_ix().await;
    init_ix.accounts[1] = AccountMeta::new_readonly(authority.pubkey(), false);

    let tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &new_authority],
        ctx.last_blockhash,
    );

    let transaction_error: TransportError = ctx
        .banks_client
        .process_transaction(tx)
        .await
        .err()
        .expect("Should have errored")
        .into();

    match transaction_error {
        TransportError::TransactionError(TransactionError::InstructionError(_, error)) => {
            assert_eq!(
                error,
                InstructionError::Custom(StakeDepositInterceptorError::SignatureMissing as u32)
            );
        }
        _ => panic!("Wrong error"),
    };
}

#[tokio::test]
async fn test_fail_authority_incorrect() {
    let (mut ctx, _stake_pool_accounts, _authority, new_authority, mut init_ix) =
        setup_with_ix().await;
    let bad_authority = Keypair::new();
    init_ix.accounts[1] = AccountMeta::new_readonly(bad_authority.pubkey(), true);

    let tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &bad_authority, &new_authority],
        ctx.last_blockhash,
    );

    let transaction_error: TransportError = ctx
        .banks_client
        .process_transaction(tx)
        .await
        .err()
        .expect("Should have errored")
        .into();

    match transaction_error {
        TransportError::TransactionError(TransactionError::InstructionError(_, error)) => {
            assert_eq!(
                error,
                InstructionError::Custom(StakeDepositInterceptorError::InvalidAuthority as u32)
            );
        }
        _ => panic!("Wrong error"),
    };
}

#[tokio::test]
async fn test_fail_invalid_stake_deposit_authority_address() {
    let (mut ctx, stake_pool_accounts, authority, new_authority, mut init_ix) =
        setup_with_ix().await;
    let bad_account = Pubkey::new_unique();
    let (deposit_stake_authority_pda, _bump) = derive_stake_pool_deposit_stake_authority(
        &stake_deposit_interceptor::id(),
        &stake_pool_accounts.stake_pool,
    );
    let original_deposit_stake_authority = ctx
        .banks_client
        .get_account(deposit_stake_authority_pda)
        .await
        .unwrap()
        .unwrap();
    let mut bad_deposit_stake_authority = AccountSharedData::new(
        original_deposit_stake_authority.lamports,
        original_deposit_stake_authority.data.len(),
        &original_deposit_stake_authority.owner,
    );
    bad_deposit_stake_authority.set_data_from_slice(&original_deposit_stake_authority.data);
    ctx.set_account(&bad_account, &bad_deposit_stake_authority);
    init_ix.accounts[0] = AccountMeta::new(bad_account, false);

    let tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority, &new_authority],
        ctx.last_blockhash,
    );

    let transaction_error: TransportError = ctx
        .banks_client
        .process_transaction(tx)
        .await
        .err()
        .expect("Should have errored")
        .into();

    match transaction_error {
        TransportError::TransactionError(TransactionError::InstructionError(_, error)) => {
            assert_eq!(
                error,
                InstructionError::Custom(StakeDepositInterceptorError::InvalidStakePoolDepositStakeAuthority as u32)
            );
        }
        _ => panic!("Wrong error"),
    };
}

#[tokio::test]
async fn test_fail_new_authority_not_signer() {
    let (mut ctx, _stake_pool_accounts, authority, new_authority, mut init_ix) =
        setup_with_ix().await;
    init_ix.accounts[2] = AccountMeta::new_readonly(new_authority.pubkey(), false);

    let tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );

    let transaction_error: TransportError = ctx
        .banks_client
        .process_transaction(tx)
        .await
        .err()
        .expect("Should have errored")
        .into();

    match transaction_error {
        TransportError::TransactionError(TransactionError::InstructionError(_, error)) => {
            assert_eq!(
                error,
                InstructionError::Custom(StakeDepositInterceptorError::SignatureMissing as u32)
            );
        }
        _ => panic!("Wrong error"),
    };
}
