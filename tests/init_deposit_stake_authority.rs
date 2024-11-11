mod helpers;

use helpers::{program_test_context_with_stake_pool_state, StakePoolAccounts};
use jito_bytemuck::AccountDeserialize;
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    instruction::{AccountMeta, Instruction, InstructionError},
    program_pack::Pack,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::{Transaction, TransactionError},
    transport::TransportError,
};
use spl_associated_token_account::get_associated_token_address;
use stake_deposit_interceptor::{
    error::StakeDepositInterceptorError, instruction::derive_stake_pool_deposit_stake_authority,
    state::StakePoolDepositStakeAuthority,
};

#[tokio::test]
async fn test_init_deposit_stake_authority() {
    let (mut ctx, stake_pool_accounts) = program_test_context_with_stake_pool_state().await;

    let fee_wallet = Keypair::new();
    let authority = Keypair::new();
    let cool_down_period = 100;
    let initial_fee_rate = 20;
    let init_ix =
        stake_deposit_interceptor::instruction::create_init_deposit_stake_authority_instruction(
            &stake_deposit_interceptor::id(),
            &ctx.payer.pubkey(),
            &stake_pool_accounts.stake_pool,
            &stake_pool_accounts.pool_mint,
            &ctx.payer.pubkey(),
            &spl_stake_pool::id(),
            &spl_token::id(),
            &fee_wallet.pubkey(),
            cool_down_period,
            initial_fee_rate,
            &authority.pubkey(),
        );

    let tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );

    ctx.banks_client.process_transaction(tx).await.unwrap();

    let (deposit_stake_authority_pubkey, _bump_seed) = derive_stake_pool_deposit_stake_authority(
        &stake_deposit_interceptor::ID,
        &stake_pool_accounts.stake_pool,
    );
    let vault_ata = get_associated_token_address(
        &deposit_stake_authority_pubkey,
        &stake_pool_accounts.pool_mint,
    );

    let account = ctx
        .banks_client
        .get_account(deposit_stake_authority_pubkey)
        .await
        .unwrap()
        .unwrap();

    let vault_account = ctx
        .banks_client
        .get_account(vault_ata)
        .await
        .unwrap()
        .unwrap();

    let deposit_stake_authority =
        StakePoolDepositStakeAuthority::try_from_slice_unchecked(&account.data.as_slice()).unwrap();
    let vault_token_account =
        spl_token::state::Account::unpack(vault_account.data.as_slice()).unwrap();
    assert_eq!(vault_token_account.mint, stake_pool_accounts.pool_mint);
    assert_eq!(vault_token_account.amount, 0);
    assert_eq!(vault_token_account.owner, deposit_stake_authority_pubkey);

    assert_eq!(deposit_stake_authority.authority, authority.pubkey());
    let actual_cool_down_period: u64 = deposit_stake_authority.cool_down_period.into();
    let actual_initial_fee_rate: u32 = deposit_stake_authority.inital_fee_rate.into();
    assert_eq!(actual_cool_down_period, cool_down_period);
    assert_eq!(actual_initial_fee_rate, initial_fee_rate);
    assert_eq!(
        deposit_stake_authority.stake_pool,
        stake_pool_accounts.stake_pool
    );
    assert_eq!(
        deposit_stake_authority.pool_mint,
        stake_pool_accounts.pool_mint
    );
    assert_eq!(
        deposit_stake_authority.stake_pool_program_id,
        spl_stake_pool::id()
    );
    assert_eq!(deposit_stake_authority.fee_wallet, fee_wallet.pubkey());
    assert_eq!(deposit_stake_authority.vault, vault_ata);
}

async fn setup_with_ix() -> (ProgramTestContext, StakePoolAccounts, Keypair, Instruction) {
    let (ctx, stake_pool_accounts) = program_test_context_with_stake_pool_state().await;

    let fee_wallet = Keypair::new();
    let authority = Keypair::new();
    let cool_down_period = 100;
    let initial_fee_rate = 20;
    let ix =
        stake_deposit_interceptor::instruction::create_init_deposit_stake_authority_instruction(
            &stake_deposit_interceptor::id(),
            &ctx.payer.pubkey(),
            &stake_pool_accounts.stake_pool,
            &stake_pool_accounts.pool_mint,
            &ctx.payer.pubkey(),
            &spl_stake_pool::id(),
            &spl_token::id(),
            &fee_wallet.pubkey(),
            cool_down_period,
            initial_fee_rate,
            &authority.pubkey(),
        );
    (ctx, stake_pool_accounts, authority, ix)
}

#[tokio::test]
async fn test_fail_invalid_system_program() {
    let (mut ctx, _stake_pool_accounts, authority, mut init_ix) = setup_with_ix().await;
    init_ix.accounts[10] = AccountMeta::new_readonly(Pubkey::new_unique(), false);

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
            assert_eq!(error, InstructionError::IncorrectProgramId);
        }
        _ => panic!("Wrong error"),
    };
}

#[tokio::test]
async fn test_fail_authority_non_signer() {
    let (mut ctx, _stake_pool_accounts, authority, mut init_ix) = setup_with_ix().await;
    init_ix.accounts[3] = AccountMeta::new(authority.pubkey(), false);

    let tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer],
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
async fn test_fail_stakepool_manager_non_signer() {
    let (mut ctx, _stake_pool_accounts, authority, mut init_ix) = setup_with_ix().await;
    let manager = Keypair::new();
    init_ix.accounts[6] = AccountMeta::new_readonly(manager.pubkey(), false);

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

#[tokio::test]
async fn test_fail_incorrect_stakepool_program() {
    let (mut ctx, _stake_pool_accounts, authority, mut init_ix) = setup_with_ix().await;
    init_ix.accounts[7] = AccountMeta::new_readonly(Pubkey::new_unique(), false);

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
                InstructionError::Custom(StakeDepositInterceptorError::InvalidStakePool as u32)
            );
        }
        _ => panic!("Wrong error"),
    };
}

#[tokio::test]
async fn test_fail_incorrect_stakepool_manager() {
    let (mut ctx, _stake_pool_accounts, authority, mut init_ix) = setup_with_ix().await;
    let manager = Keypair::new();
    init_ix.accounts[6] = AccountMeta::new_readonly(manager.pubkey(), true);

    let tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority, &manager],
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
                InstructionError::Custom(
                    StakeDepositInterceptorError::InvalidStakePoolManager as u32
                )
            );
        }
        _ => panic!("Wrong error"),
    };
}

#[tokio::test]
async fn test_fail_incorrect_stakepool_mint() {
    let (mut ctx, _stake_pool_accounts, authority, mut init_ix) = setup_with_ix().await;
    init_ix.accounts[5] = AccountMeta::new_readonly(Pubkey::new_unique(), false);

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
                InstructionError::Custom(StakeDepositInterceptorError::InvalidStakePool as u32)
            );
        }
        _ => panic!("Wrong error"),
    };
}

#[tokio::test]
async fn test_fail_incorrect_token_program() {
    let (mut ctx, _stake_pool_accounts, authority, mut init_ix) = setup_with_ix().await;
    init_ix.accounts[8] = AccountMeta::new_readonly(spl_token_2022::id(), false);

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
                InstructionError::Custom(StakeDepositInterceptorError::InvalidTokenProgram as u32)
            );
        }
        _ => panic!("Wrong error"),
    };
}

#[tokio::test]
async fn test_fail_incorrect_deposit_stake_authority() {
    let (mut ctx, _stake_pool_accounts, authority, mut init_ix) = setup_with_ix().await;
    init_ix.accounts[1] = AccountMeta::new(Pubkey::new_unique(), false);

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
                InstructionError::Custom(StakeDepositInterceptorError::InvalidSeeds as u32)
            );
        }
        _ => panic!("Wrong error"),
    };
}

#[tokio::test]
async fn test_fail_incorrect_vault() {
    let (mut ctx, _stake_pool_accounts, authority, mut init_ix) = setup_with_ix().await;
    init_ix.accounts[2] = AccountMeta::new(Pubkey::new_unique(), false);

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
                InstructionError::Custom(StakeDepositInterceptorError::InvalidVault as u32)
            );
        }
        _ => panic!("Wrong error"),
    };
}
