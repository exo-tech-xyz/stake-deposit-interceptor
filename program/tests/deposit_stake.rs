mod helpers;

use helpers::{
    airdrop_lamports, assert_transaction_err, clone_account_to_new_address, create_stake_account,
    create_stake_deposit_authority, create_token_account, create_validator_and_add_to_pool,
    delegate_stake_account, get_account, get_account_data_deserialized,
    program_test_context_with_stake_pool_state, stake_pool_update_all,
    update_stake_deposit_authority, StakePoolAccounts, ValidatorStakeAccount,
};
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    account::AccountSharedData,
    borsh1::try_from_slice_unchecked,
    instruction::{AccountMeta, Instruction, InstructionError},
    native_token::LAMPORTS_PER_SOL,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    stake::{self},
    system_program,
    transaction::{Transaction, TransactionError},
    transport::TransportError,
};
use spl_associated_token_account::get_associated_token_address;
use spl_pod::primitives::PodU64;
use spl_stake_pool::error::StakePoolError;
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
    Keypair,
    Keypair,
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
    let deposit_authority_base = Keypair::new();
    let (deposit_stake_authority_pubkey, _bump) = derive_stake_pool_deposit_stake_authority(
        &stake_deposit_interceptor::id(),
        &stake_pool_accounts.stake_pool,
        &deposit_authority_base.pubkey(),
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
        &deposit_authority_base,
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
    let deposit_receipt_base = Keypair::new();
    (
        ctx,
        stake_pool_accounts,
        stake_pool,
        validator_stake_accounts,
        deposit_stake_authority,
        depositor,
        depositor_stake_account,
        deposit_receipt_base,
        deposit_authority_base,
        total_staked_amount,
    )
}

#[tokio::test]
async fn test_deposit_stake() {
    let (
        mut ctx,
        stake_pool_accounts,
        stake_pool,
        validator_stake_accounts,
        deposit_stake_authority,
        depositor,
        depositor_stake_account,
        deposit_receipt_base,
        deposit_authority_base,
        total_staked_amount,
    ) = setup().await;

    let (deposit_stake_authority_pubkey, _bump_seed) = derive_stake_pool_deposit_stake_authority(
        &stake_deposit_interceptor::id(),
        &stake_pool_accounts.stake_pool,
        &deposit_authority_base.pubkey(),
    );

    // Actually test DepositStake
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
            &deposit_receipt_base.pubkey(),
            &deposit_authority_base.pubkey(),
        );

    let tx = Transaction::new_signed_with_payer(
        &deposit_stake_instructions,
        Some(&depositor.pubkey()),
        &[&depositor, &deposit_receipt_base],
        ctx.last_blockhash,
    );

    ctx.banks_client.process_transaction(tx).await.unwrap();

    let vault_account = get_account(&mut ctx.banks_client, &deposit_stake_authority.vault).await;
    let vault = spl_token::state::Account::unpack(&vault_account.data).unwrap();

    let pool_tokens_amount = spl_stake_pool::state::StakePool::calc_pool_tokens_for_deposit(
        &stake_pool,
        total_staked_amount,
    )
    .unwrap();

    // assert LST was transfer to the vault
    assert_eq!(vault.amount, pool_tokens_amount);

    // Assert DepositReceipt has correct data.
    let (deposit_receipt_pda, bump_seed) = derive_stake_deposit_receipt(
        &stake_deposit_interceptor::id(),
        &stake_pool_accounts.stake_pool,
        &deposit_receipt_base.pubkey(),
    );
    let deposit_receipt = get_account_data_deserialized::<DepositReceipt>(
        &mut ctx.banks_client,
        &deposit_receipt_pda,
    )
    .await;
    assert_eq!(deposit_receipt.owner, depositor.pubkey());
    assert_eq!(deposit_receipt.base, deposit_receipt_base.pubkey());
    assert_eq!(deposit_receipt.stake_pool, stake_pool_accounts.stake_pool);
    assert_eq!(
        deposit_receipt.stake_pool_deposit_stake_authority,
        deposit_stake_authority_pubkey
    );
    assert_eq!(deposit_receipt.bump_seed, bump_seed);
    assert_eq!(deposit_receipt.lst_amount, PodU64::from(pool_tokens_amount));
    assert_eq!(
        deposit_receipt.cool_down_seconds,
        deposit_stake_authority.cool_down_seconds
    );
    assert_eq!(
        deposit_receipt.initial_fee_bps,
        deposit_stake_authority.inital_fee_bps
    );
    let deposit_time: u64 = deposit_receipt.deposit_time.into();
    assert!(deposit_time > 0);
}

#[tokio::test]
async fn success_error_with_slippage() {
    let (
        mut ctx,
        stake_pool_accounts,
        stake_pool,
        validator_stake_accounts,
        deposit_stake_authority,
        depositor,
        depositor_stake_account,
        deposit_receipt_base,
        deposit_authority_base,
        total_staked_amount,
    ) = setup().await;

    let pool_tokens_amount = spl_stake_pool::state::StakePool::calc_pool_tokens_for_deposit(
        &stake_pool,
        total_staked_amount,
    )
    .unwrap();

    let deposit_stake_with_slippage_instructions =
        stake_deposit_interceptor::instruction::create_deposit_stake_with_slippage_instruction(
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
            &deposit_receipt_base.pubkey(),
            &deposit_authority_base.pubkey(),
            pool_tokens_amount + 1,
        );

    let tx = Transaction::new_signed_with_payer(
        &deposit_stake_with_slippage_instructions,
        Some(&depositor.pubkey()),
        &[&depositor, &deposit_receipt_base],
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
                InstructionError::Custom(StakePoolError::ExceededSlippage as u32)
            );
        }
        _ => panic!("Wrong error"),
    };

    let deposit_stake_with_slippage_instructions =
        stake_deposit_interceptor::instruction::create_deposit_stake_with_slippage_instruction(
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
            &deposit_receipt_base.pubkey(),
            &deposit_authority_base.pubkey(),
            pool_tokens_amount,
        );

    let tx = Transaction::new_signed_with_payer(
        &deposit_stake_with_slippage_instructions,
        Some(&depositor.pubkey()),
        &[&depositor, &deposit_receipt_base],
        ctx.last_blockhash,
    );

    ctx.banks_client.process_transaction(tx).await.unwrap();
}

async fn setup_with_ix() -> (
    ProgramTestContext,
    StakePoolAccounts,
    Keypair,
    Pubkey,
    Pubkey,
    Keypair,
    Vec<Instruction>,
) {
    let (
        ctx,
        stake_pool_accounts,
        _stake_pool,
        validator_stake_accounts,
        deposit_stake_authority,
        depositor,
        depositor_stake_account,
        deposit_receipt_base,
        deposit_authority_base,
        _total_staked_amount,
    ) = setup().await;

    // Actually test DepositStake
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
            &deposit_receipt_base.pubkey(),
            &deposit_authority_base.pubkey(),
        );

    let (deposit_stake_authority_pubkey, _bump) = derive_stake_pool_deposit_stake_authority(
        &stake_deposit_interceptor::id(),
        &stake_pool_accounts.stake_pool,
        &deposit_authority_base.pubkey(),
    );
    let (deposit_receipt_pda, _bump_seed) = derive_stake_deposit_receipt(
        &stake_deposit_interceptor::id(),
        &stake_pool_accounts.stake_pool,
        &deposit_receipt_base.pubkey(),
    );

    (
        ctx,
        stake_pool_accounts,
        deposit_receipt_base,
        deposit_receipt_pda,
        deposit_stake_authority_pubkey,
        depositor,
        deposit_stake_instructions,
    )
}

#[tokio::test]
async fn test_fail_invalid_system_program() {
    let (
        mut ctx,
        _stake_pool_accounts,
        deposit_receipt_base,
        _deposit_receipt_pda,
        _deposit_stake_authority_pubkey,
        depositor,
        mut instructions,
    ) = setup_with_ix().await;
    instructions[2].accounts[19] = AccountMeta::new_readonly(Pubkey::new_unique(), false);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&depositor.pubkey()),
        &[&depositor, &deposit_receipt_base],
        ctx.last_blockhash,
    );

    assert_transaction_err(&mut ctx, tx, InstructionError::IncorrectProgramId).await;
}

#[tokio::test]
async fn test_fail_invalid_stake_pool_program() {
    let (
        mut ctx,
        _stake_pool_accounts,
        deposit_receipt_base,
        _deposit_receipt_pda,
        _deposit_stake_authority_pubkey,
        depositor,
        mut instructions,
    ) = setup_with_ix().await;
    instructions[2].accounts[1] = AccountMeta::new_readonly(Pubkey::new_unique(), false);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&depositor.pubkey()),
        &[&depositor, &deposit_receipt_base],
        ctx.last_blockhash,
    );

    assert_transaction_err(
        &mut ctx,
        tx,
        InstructionError::Custom(StakeDepositInterceptorError::InvalidStakePoolProgram as u32),
    )
    .await;
}

#[tokio::test]
async fn test_fail_invalid_deposit_stake_authority_owner() {
    let (
        mut ctx,
        _stake_pool_accounts,
        deposit_receipt_base,
        _deposit_receipt_pda,
        _deposit_stake_authority_pubkey,
        depositor,
        mut instructions,
    ) = setup_with_ix().await;
    instructions[2].accounts[5] = AccountMeta::new_readonly(Pubkey::new_unique(), false);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&depositor.pubkey()),
        &[&depositor, &deposit_receipt_base],
        ctx.last_blockhash,
    );

    assert_transaction_err(&mut ctx, tx, InstructionError::IncorrectProgramId).await;
}

#[tokio::test]
async fn test_fail_invalid_stake_deposit_authority_address() {
    let (
        mut ctx,
        _stake_pool_accounts,
        deposit_receipt_base,
        _deposit_receipt_pda,
        deposit_stake_authority_pubkey,
        depositor,
        mut instructions,
    ) = setup_with_ix().await;
    let bad_account = clone_account_to_new_address(&mut ctx, &deposit_stake_authority_pubkey).await;
    instructions[2].accounts[5] = AccountMeta::new_readonly(bad_account, false);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&depositor.pubkey()),
        &[&depositor, &deposit_receipt_base],
        ctx.last_blockhash,
    );

    assert_transaction_err(
        &mut ctx,
        tx,
        InstructionError::Custom(
            StakeDepositInterceptorError::InvalidStakePoolDepositStakeAuthority as u32,
        ),
    )
    .await;
}

#[tokio::test]
async fn test_fail_invalid_pool_token_account() {
    let (
        mut ctx,
        stake_pool_accounts,
        deposit_receipt_base,
        _deposit_receipt_pda,
        deposit_stake_authority_pubkey,
        depositor,
        mut instructions,
    ) = setup_with_ix().await;
    let vault_token_account = get_associated_token_address(
        &deposit_stake_authority_pubkey,
        &stake_pool_accounts.pool_mint,
    );
    let bad_account = clone_account_to_new_address(&mut ctx, &vault_token_account).await;
    instructions[2].accounts[11] = AccountMeta::new_readonly(bad_account, false);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&depositor.pubkey()),
        &[&depositor, &deposit_receipt_base],
        ctx.last_blockhash,
    );

    assert_transaction_err(
        &mut ctx,
        tx,
        InstructionError::Custom(StakeDepositInterceptorError::InvalidVault as u32),
    )
    .await;
}

#[tokio::test]
async fn test_fail_invalid_deposit_receipt_owner() {
    let (
        mut ctx,
        _stake_pool_accounts,
        deposit_receipt_base,
        deposit_receipt_pda,
        _deposit_stake_authority_pubkey,
        depositor,
        instructions,
    ) = setup_with_ix().await;
    let bad_account = AccountSharedData::new(1, 0, &stake_deposit_interceptor::id());
    ctx.set_account(&deposit_receipt_pda, &bad_account);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&depositor.pubkey()),
        &[&depositor, &deposit_receipt_base],
        ctx.last_blockhash,
    );

    assert_transaction_err(&mut ctx, tx.clone(), InstructionError::InvalidAccountOwner).await;
}

#[tokio::test]
async fn test_fail_deposit_receipt_not_empty() {
    let (
        mut ctx,
        _stake_pool_accounts,
        deposit_receipt_base,
        deposit_receipt_pda,
        _deposit_stake_authority_pubkey,
        depositor,
        instructions,
    ) = setup_with_ix().await;

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&depositor.pubkey()),
        &[&depositor, &deposit_receipt_base],
        ctx.last_blockhash,
    );

    let bad_account = AccountSharedData::new(1, 1, &system_program::id());
    ctx.set_account(&deposit_receipt_pda, &bad_account);
    assert_transaction_err(
        &mut ctx,
        tx.clone(),
        InstructionError::AccountAlreadyInitialized,
    )
    .await;
}

#[tokio::test]
async fn test_fail_invalid_deposit_receipt() {
    let (
        mut ctx,
        _stake_pool_accounts,
        deposit_receipt_base,
        _deposit_receipt_pda,
        _deposit_stake_authority_pubkey,
        depositor,
        mut instructions,
    ) = setup_with_ix().await;
    instructions[2].accounts[2] = AccountMeta::new(Pubkey::new_unique(), false);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&depositor.pubkey()),
        &[&depositor, &deposit_receipt_base],
        ctx.last_blockhash,
    );

    assert_transaction_err(
        &mut ctx,
        tx,
        InstructionError::Custom(StakeDepositInterceptorError::InvalidSeeds as u32),
    )
    .await;
}
