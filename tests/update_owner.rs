mod helpers;

use helpers::{
    airdrop_lamports, clone_account_to_new_address, create_stake_account,
    create_stake_deposit_authority, create_token_account, create_validator_and_add_to_pool,
    delegate_stake_account, get_account_data_deserialized,
    program_test_context_with_stake_pool_state, stake_pool_update_all,
    update_stake_deposit_authority, StakePoolAccounts, ValidatorStakeAccount,
};
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    borsh1::try_from_slice_unchecked,
    instruction::{AccountMeta, Instruction, InstructionError},
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    stake::{self},
    transaction::{Transaction, TransactionError},
    transport::TransportError,
};
use stake_deposit_interceptor::{
    error::StakeDepositInterceptorError,
    instruction::{derive_stake_deposit_receipt, derive_stake_pool_deposit_stake_authority},
    state::{DepositReceipt, StakePoolDepositStakeAuthority},
};

async fn setup() -> (
    ProgramTestContext,
    StakePoolAccounts,
    spl_stake_pool::state::StakePool,
    ValidatorStakeAccount,
    StakePoolDepositStakeAuthority,
    Keypair,
    Pubkey,
    Pubkey,
    u64,
) {
    let (mut ctx, stake_pool_accounts) = program_test_context_with_stake_pool_state().await;
    let rent = ctx.banks_client.get_rent().await.unwrap();
    let stake_pool_account = ctx
        .banks_client
        .get_account(stake_pool_accounts.stake_pool)
        .await
        .unwrap()
        .unwrap();
    let stake_pool =
        try_from_slice_unchecked::<spl_stake_pool::state::StakePool>(&stake_pool_account.data)
            .unwrap();
    let (deposit_stake_authority_pubkey, _bump) = derive_stake_pool_deposit_stake_authority(
        &stake_deposit_interceptor::id(),
        &stake_pool_accounts.stake_pool,
    );
    // Set the StakePool's stake_deposit_authority to the interceptor program's PDA
    update_stake_deposit_authority(
        &mut ctx.banks_client,
        &stake_pool_accounts,
        &deposit_stake_authority_pubkey,
        &ctx.payer,
        ctx.last_blockhash,
    )
    .await;
    // Add a validator to the stake_pool
    let validator_stake_accounts =
        create_validator_and_add_to_pool(&mut ctx, &stake_pool_accounts).await;

    let authority = Keypair::new();
    create_stake_deposit_authority(
        &mut ctx,
        &stake_pool_accounts.stake_pool,
        &stake_pool.pool_mint,
        &authority,
        None,
    )
    .await;

    let depositor = Keypair::new();
    airdrop_lamports(&mut ctx, &depositor.pubkey(), 10 * LAMPORTS_PER_SOL).await;

    // Create "Depositor" owned stake account
    let authorized = stake::state::Authorized {
        staker: depositor.pubkey(),
        withdrawer: depositor.pubkey(),
    };
    let lockup = stake::state::Lockup::default();
    let stake_amount = 2 * LAMPORTS_PER_SOL;
    let total_staked_amount =
        rent.minimum_balance(std::mem::size_of::<stake::state::StakeStateV2>()) + stake_amount;
    let depositor_stake_account = create_stake_account(
        &mut ctx.banks_client,
        &depositor,
        &authorized,
        &lockup,
        stake_amount,
        ctx.last_blockhash,
    )
    .await;

    // Create a TokenAccount for the "Depositor" of the StakePool's `pool_mint`.
    let _depositor_lst_account = create_token_account(
        &mut ctx,
        &depositor.pubkey(),
        &stake_pool_accounts.pool_mint,
    )
    .await;

    // Delegate the "Depositor" stake account to a validator from
    // the relevant StakePool.
    delegate_stake_account(
        &mut ctx.banks_client,
        &depositor,
        &ctx.last_blockhash,
        &depositor_stake_account,
        &depositor,
        &validator_stake_accounts.vote.pubkey(),
    )
    .await;

    // Fast forward to next epoch so stake is active
    let first_normal_slot = ctx.genesis_config().epoch_schedule.first_normal_slot;
    ctx.warp_to_slot(first_normal_slot + 1).unwrap();

    // Update relevant stake_pool state
    stake_pool_update_all(
        &mut ctx.banks_client,
        &ctx.payer,
        &stake_pool_accounts,
        &ctx.last_blockhash,
        false,
    )
    .await;

    // Get latest `StakePoolDepositStakeAuthority``
    let deposit_stake_authority = get_account_data_deserialized::<StakePoolDepositStakeAuthority>(
        &mut ctx.banks_client,
        &deposit_stake_authority_pubkey,
    )
    .await;

    // Generate a random Pubkey as seed for DepositReceipt PDA.
    let base = Pubkey::new_unique();
    let deposit_stake_instructions =
        stake_deposit_interceptor::instruction::create_deposit_stake_instruction(
            &stake_deposit_interceptor::id(),
            &depositor.pubkey(),
            &spl_stake_pool::id(),
            &stake_pool_accounts.stake_pool,
            &stake_pool_accounts.validator_list,
            &stake_pool_accounts.withdraw_authority,
            &depositor_stake_account,
            &depositor.pubkey(),
            &validator_stake_accounts.stake_account,
            &stake_pool_accounts.reserve_stake_account,
            &deposit_stake_authority.vault,
            &stake_pool_accounts.pool_fee_account,
            &stake_pool_accounts.pool_fee_account,
            &stake_pool_accounts.pool_mint,
            &spl_token::id(),
            &base,
        );

    let tx = Transaction::new_signed_with_payer(
        &deposit_stake_instructions,
        Some(&depositor.pubkey()),
        &[&depositor],
        ctx.last_blockhash,
    );

    ctx.banks_client.process_transaction(tx).await.unwrap();
    (
        ctx,
        stake_pool_accounts,
        stake_pool,
        validator_stake_accounts,
        deposit_stake_authority,
        depositor,
        depositor_stake_account,
        base,
        total_staked_amount,
    )
}

#[tokio::test]
async fn success() {
    let (
        mut ctx,
        stake_pool_accounts,
        _stake_pool,
        _validator_stake_accounts,
        _deposit_stake_authority,
        depositor,
        _depositor_stake_account,
        base,
        _total_staked_amount,
    ) = setup().await;

    let (deposit_receipt_pda, _bump_seed) = derive_stake_deposit_receipt(
        &stake_deposit_interceptor::id(),
        &depositor.pubkey(),
        &stake_pool_accounts.stake_pool,
        &base,
    );

    let new_owner = Pubkey::new_unique();

    // Update owner of DepositReceipt
    let ix = stake_deposit_interceptor::instruction::create_change_deposit_receipt_owner(
        &stake_deposit_interceptor::id(),
        &deposit_receipt_pda,
        &depositor.pubkey(),
        &new_owner,
    );

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&depositor.pubkey()),
        &[&depositor],
        ctx.last_blockhash,
    );

    ctx.banks_client.process_transaction(tx).await.unwrap();

    let deposit_receipt = get_account_data_deserialized::<DepositReceipt>(
        &mut ctx.banks_client,
        &deposit_receipt_pda,
    )
    .await;
    assert_eq!(deposit_receipt.owner, new_owner);
}

async fn setup_with_ix() -> (ProgramTestContext, Keypair, Pubkey, Instruction) {
    let (
        ctx,
        stake_pool_accounts,
        _stake_pool,
        _validator_stake_accounts,
        _deposit_stake_authority,
        depositor,
        _depositor_stake_account,
        base,
        _total_staked_amount,
    ) = setup().await;

    let (deposit_receipt_pda, _bump_seed) = derive_stake_deposit_receipt(
        &stake_deposit_interceptor::id(),
        &depositor.pubkey(),
        &stake_pool_accounts.stake_pool,
        &base,
    );

    let new_owner = Pubkey::new_unique();

    // Update owner of DepositReceipt
    let ix = stake_deposit_interceptor::instruction::create_change_deposit_receipt_owner(
        &stake_deposit_interceptor::id(),
        &deposit_receipt_pda,
        &depositor.pubkey(),
        &new_owner,
    );
    (ctx, depositor, deposit_receipt_pda, ix)
}

#[tokio::test]
async fn test_fail_owner_not_signer() {
    let (mut ctx, depositor, _deposit_receipt_pda, mut ix) = setup_with_ix().await;
    ix.accounts[1] = AccountMeta::new(depositor.pubkey(), false);

    let tx = Transaction::new_signed_with_payer(
        &[ix],
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
async fn test_fail_invalid_deposit_receipt_address() {
    let (mut ctx, depositor, deposit_receipt_pda, mut ix) = setup_with_ix().await;
    let bad_account = clone_account_to_new_address(&mut ctx, &deposit_receipt_pda).await;
    ix.accounts[0] = AccountMeta::new(bad_account, false);

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &depositor],
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
                    StakeDepositInterceptorError::InvalidDepositReceipt as u32
                )
            );
        }
        _ => panic!("Wrong error"),
    };
}

#[tokio::test]
async fn test_fail_invalid_owner() {
    let (mut ctx, _depositor, _deposit_receipt_pda, mut ix) = setup_with_ix().await;
    let bad_owner = Keypair::new();
    ix.accounts[1] = AccountMeta::new_readonly(bad_owner.pubkey(), true);

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &bad_owner],
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
                    StakeDepositInterceptorError::InvalidDepositReceiptOwner as u32
                )
            );
        }
        _ => panic!("Wrong error"),
    };
}
