mod helpers;

use std::{mem, ops::Add};

use helpers::{
    airdrop_lamports, assert_transaction_err, clone_account_to_new_address, create_stake_account,
    create_stake_deposit_authority, create_token_account, create_validator_and_add_to_pool,
    delegate_stake_account, get_account, get_account_data_deserialized,
    program_test_context_with_stake_pool_state, set_clock_time, stake_pool_update_all,
    update_stake_deposit_authority, StakePoolAccounts, ValidatorStakeAccount,
};
use jito_bytemuck::{AccountDeserialize, Discriminator};
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    account::AccountSharedData,
    borsh1::try_from_slice_unchecked,
    clock::Clock,
    instruction::{AccountMeta, Instruction, InstructionError},
    native_token::LAMPORTS_PER_SOL,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    stake::{self},
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token_2022::state::Account;
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
    Pubkey,
    Keypair,
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

    let fee_wallet = Keypair::new();

    let authority = Keypair::new();
    create_stake_deposit_authority(
        &mut ctx,
        &stake_pool_accounts.stake_pool,
        &stake_pool.pool_mint,
        &authority,
        Some(&fee_wallet.pubkey()),
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
    let depositor_pool_token_account = create_token_account(
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
        depositor_pool_token_account,
        fee_wallet,
    )
}

#[tokio::test]
async fn test_success_claim_pool_tokens() {
    let (
        mut ctx,
        stake_pool_accounts,
        stake_pool,
        _validator_stake_accounts,
        deposit_stake_authority,
        depositor,
        _depositor_stake_account,
        base,
        _total_staked_amount,
        depositor_pool_token_account,
        fee_wallet,
    ) = setup().await;

    let (deposit_stake_authority_pubkey, _bump_seed) = derive_stake_pool_deposit_stake_authority(
        &stake_deposit_interceptor::id(),
        &stake_pool_accounts.stake_pool,
    );
    let (deposit_receipt_pda, _bump_seed) = derive_stake_deposit_receipt(
        &stake_deposit_interceptor::id(),
        &depositor.pubkey(),
        &stake_pool_accounts.stake_pool,
        &base,
    );

    let deposit_receipt = get_account_data_deserialized::<DepositReceipt>(
        &mut ctx.banks_client,
        &deposit_receipt_pda,
    )
    .await;

    let fee_token_account =
        get_associated_token_address(&fee_wallet.pubkey(), &stake_pool_accounts.pool_mint);

    let create_fee_token_account_ix = create_associated_token_account(
        &depositor.pubkey(),
        &fee_wallet.pubkey(),
        &stake_pool_accounts.pool_mint,
        &spl_token::id(),
    );

    let ix = stake_deposit_interceptor::instruction::create_claim_pool_tokens_instruction(
        &stake_deposit_interceptor::id(),
        &deposit_receipt_pda,
        &depositor.pubkey(),
        &deposit_stake_authority.vault,
        &depositor_pool_token_account,
        &fee_token_account,
        &deposit_stake_authority_pubkey,
        &stake_pool.pool_mint,
        &spl_token::id(),
        false,
    );

    let clock: Clock = ctx.banks_client.get_sysvar().await.unwrap();
    let half_cool_down = u64::from(deposit_receipt.cool_down_period).saturating_div(2);
    let clock_time = clock.unix_timestamp + half_cool_down as i64;
    set_clock_time(&mut ctx, clock_time).await;

    let tx = Transaction::new_signed_with_payer(
        &[create_fee_token_account_ix, ix],
        Some(&depositor.pubkey()),
        &[&depositor],
        ctx.last_blockhash,
    );

    ctx.banks_client.process_transaction(tx).await.unwrap();

    let fee_amount = deposit_receipt.calculate_fee_amount(clock_time);
    let user_amount = u64::from(deposit_receipt.lst_amount) - fee_amount;

    // Destination token account should have received pool tokens
    let destination_token_account_info =
        get_account(&mut ctx.banks_client, &depositor_pool_token_account).await;
    let destination_token_account =
        Account::unpack(&destination_token_account_info.data.as_slice()).unwrap();
    assert_eq!(destination_token_account.amount, user_amount);

    // Fees should have been paid
    let fee_token_account_info = get_account(&mut ctx.banks_client, &fee_token_account).await;
    let fee_token_account = Account::unpack(&fee_token_account_info.data.as_slice()).unwrap();
    assert_eq!(fee_token_account.amount, fee_amount,);

    // DepositReceipt account should have been closed
    let deposit_receipt_account = ctx
        .banks_client
        .get_account(deposit_receipt_pda)
        .await
        .unwrap();
    assert!(deposit_receipt_account.is_none());
}

async fn setup_with_ix() -> (
    ProgramTestContext,
    StakePoolAccounts,
    Keypair,
    Pubkey,
    Vec<Instruction>,
) {
    let (
        ctx,
        stake_pool_accounts,
        stake_pool,
        _validator_stake_accounts,
        deposit_stake_authority,
        depositor,
        _depositor_stake_account,
        base,
        _total_staked_amount,
        depositor_pool_token_account,
        fee_wallet,
    ) = setup().await;

    let (deposit_stake_authority_pubkey, _bump_seed) = derive_stake_pool_deposit_stake_authority(
        &stake_deposit_interceptor::id(),
        &stake_pool_accounts.stake_pool,
    );
    let (deposit_receipt_pda, _bump_seed) = derive_stake_deposit_receipt(
        &stake_deposit_interceptor::id(),
        &depositor.pubkey(),
        &stake_pool_accounts.stake_pool,
        &base,
    );

    let fee_token_account =
        get_associated_token_address(&fee_wallet.pubkey(), &stake_pool_accounts.pool_mint);

    let create_fee_token_account_ix = create_associated_token_account(
        &depositor.pubkey(),
        &fee_wallet.pubkey(),
        &stake_pool_accounts.pool_mint,
        &spl_token::id(),
    );

    let ix = stake_deposit_interceptor::instruction::create_claim_pool_tokens_instruction(
        &stake_deposit_interceptor::id(),
        &deposit_receipt_pda,
        &depositor.pubkey(),
        &deposit_stake_authority.vault,
        &depositor_pool_token_account,
        &fee_token_account,
        &deposit_stake_authority_pubkey,
        &stake_pool.pool_mint,
        &spl_token::id(),
        true,
    );
    (
        ctx,
        stake_pool_accounts,
        depositor,
        deposit_receipt_pda,
        vec![create_fee_token_account_ix, ix],
    )
}

#[tokio::test]
async fn test_success_permissionless_claim() {
    let (mut ctx, _stake_pool_accounts, _depositor, deposit_receipt_pda, mut instructions) =
        setup_with_ix().await;
    // update fee token account funder to ctx.payer
    instructions[0].accounts[0] = AccountMeta::new(ctx.payer.pubkey(), true);
    let destination_token_account = instructions[1].accounts[3].pubkey;
    let deposit_receipt = get_account_data_deserialized::<DepositReceipt>(
        &mut ctx.banks_client,
        &deposit_receipt_pda,
    )
    .await;

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer],
        ctx.last_blockhash,
    );

    let clock: Clock = ctx.banks_client.get_sysvar().await.unwrap();
    let clock_time =
        clock.unix_timestamp + u64::from(deposit_receipt.cool_down_period).add(10) as i64;
    set_clock_time(&mut ctx, clock_time).await;

    ctx.banks_client.process_transaction(tx).await.unwrap();

    let user_amount = u64::from(deposit_receipt.lst_amount);
    let destination_token_account_info =
        get_account(&mut ctx.banks_client, &destination_token_account).await;
    let destination_token_account =
        Account::unpack(&destination_token_account_info.data.as_slice()).unwrap();
    assert_eq!(destination_token_account.amount, user_amount);
}

#[tokio::test]
async fn test_fail_permissionless_claim_during_cool_down() {
    let (mut ctx, _stake_pool_accounts, depositor, _deposit_receipt_pda, mut instructions) =
        setup_with_ix().await;
    // update fee token account funder to ctx.payer
    instructions[0].accounts[0] = AccountMeta::new(ctx.payer.pubkey(), true);
    // Update instruction to not require owner signature
    instructions[1].accounts[1] = AccountMeta::new(depositor.pubkey(), false);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer],
        ctx.last_blockhash,
    );

    assert_transaction_err(
        &mut ctx,
        tx,
        InstructionError::Custom(StakeDepositInterceptorError::ActiveCooldown as u32),
    )
    .await;
}

#[tokio::test]
async fn test_fail_invalid_system_program() {
    let (mut ctx, _stake_pool_accounts, depositor, _deposit_receipt_pda, mut instructions) =
        setup_with_ix().await;
    instructions[1].accounts[8] = AccountMeta::new_readonly(Pubkey::new_unique(), false);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&depositor.pubkey()),
        &[&depositor],
        ctx.last_blockhash,
    );

    assert_transaction_err(&mut ctx, tx, InstructionError::IncorrectProgramId).await;
}

#[tokio::test]
async fn test_fail_invalid_owner() {
    let (mut ctx, _stake_pool_accounts, depositor, _deposit_receipt_pda, mut instructions) =
        setup_with_ix().await;
    let bad_owner = Keypair::new();
    instructions[1].accounts[1] = AccountMeta::new(bad_owner.pubkey(), true);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&depositor.pubkey()),
        &[&depositor, &bad_owner],
        ctx.last_blockhash,
    );

    assert_transaction_err(
        &mut ctx,
        tx,
        InstructionError::Custom(StakeDepositInterceptorError::InvalidDepositReceiptOwner as u32),
    )
    .await;
}

#[tokio::test]
async fn test_fail_invalid_stake_deposit_authority_owner() {
    let (mut ctx, stake_pool_accounts, depositor, _deposit_receipt_pda, instructions) =
        setup_with_ix().await;
    let (deposit_stake_authority_pubkey, _bump_seed) = derive_stake_pool_deposit_stake_authority(
        &stake_deposit_interceptor::id(),
        &stake_pool_accounts.stake_pool,
    );
    // Set the owner of the `StakePoolDepositStakeAuthority` to a bad pubkey.
    let original = ctx
        .banks_client
        .get_account(deposit_stake_authority_pubkey)
        .await
        .unwrap()
        .unwrap();
    const ACCOUNT_SIZE: usize = 8 + mem::size_of::<StakePoolDepositStakeAuthority>();
    let mut bad_account =
        AccountSharedData::new(original.lamports, ACCOUNT_SIZE, &Pubkey::new_unique());
    bad_account.set_data_from_slice(&original.data);
    ctx.set_account(&deposit_stake_authority_pubkey, &bad_account);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&depositor.pubkey()),
        &[&depositor],
        ctx.last_blockhash,
    );

    assert_transaction_err(&mut ctx, tx, InstructionError::IncorrectProgramId).await;
}

#[tokio::test]
async fn test_fail_invalid_stake_deposit_authority_address() {
    let (mut ctx, stake_pool_accounts, depositor, _deposit_receipt_pda, mut instructions) =
        setup_with_ix().await;
    let (deposit_stake_authority_pda, _bump) = derive_stake_pool_deposit_stake_authority(
        &stake_deposit_interceptor::id(),
        &stake_pool_accounts.stake_pool,
    );
    let bad_account = clone_account_to_new_address(&mut ctx, &deposit_stake_authority_pda).await;
    instructions[1].accounts[5] = AccountMeta::new_readonly(bad_account, false);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&depositor.pubkey()),
        &[&depositor],
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
async fn test_fail_invalid_deposit_receipt_owner() {
    let (mut ctx, _stake_pool_accounts, depositor, deposit_receipt_pda, instructions) =
        setup_with_ix().await;
    // Set the owner of the `DepositReceipt` to a bad pubkey.
    let original = ctx
        .banks_client
        .get_account(deposit_receipt_pda)
        .await
        .unwrap()
        .unwrap();
    const ACCOUNT_SIZE: usize = 8 + mem::size_of::<DepositReceipt>();
    let mut bad_account =
        AccountSharedData::new(original.lamports, ACCOUNT_SIZE, &Pubkey::new_unique());
    bad_account.set_data_from_slice(&original.data);
    ctx.set_account(&deposit_receipt_pda, &bad_account);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&depositor.pubkey()),
        &[&depositor],
        ctx.last_blockhash,
    );

    assert_transaction_err(&mut ctx, tx, InstructionError::IncorrectProgramId).await;
}

#[tokio::test]
async fn test_fail_invalid_deposit_receipt_address() {
    let (mut ctx, _stake_pool_accounts, depositor, deposit_receipt_pda, mut instructions) =
        setup_with_ix().await;
    let bad_account = clone_account_to_new_address(&mut ctx, &deposit_receipt_pda).await;
    instructions[1].accounts[0] = AccountMeta::new(bad_account, false);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &depositor],
        ctx.last_blockhash,
    );

    assert_transaction_err(
        &mut ctx,
        tx,
        InstructionError::Custom(StakeDepositInterceptorError::InvalidDepositReceipt as u32),
    )
    .await;
}

#[tokio::test]
async fn test_fail_invalid_deposit_receipt() {
    let (mut ctx, _stake_pool_accounts, depositor, deposit_receipt_pda, instructions) =
        setup_with_ix().await;
    // overwrite the `stake_pool_deposit_stake_authority` of the DepositReceipt to a bad value
    let mut original_deposit_receipt = ctx
        .banks_client
        .get_account(deposit_receipt_pda)
        .await
        .unwrap()
        .unwrap();
    const ACCOUNT_SIZE: usize = 8 + mem::size_of::<DepositReceipt>();
    let bad_deposit_receipt =
        DepositReceipt::try_from_slice_unchecked_mut(&mut original_deposit_receipt.data).unwrap();
    bad_deposit_receipt.stake_pool_deposit_stake_authority = Pubkey::new_unique();
    let mut bad_account = AccountSharedData::new(
        original_deposit_receipt.lamports,
        ACCOUNT_SIZE,
        &original_deposit_receipt.owner,
    );
    let mut data = [0u8; ACCOUNT_SIZE];
    data[0] = DepositReceipt::DISCRIMINATOR;
    borsh::to_writer(&mut data[8..], bad_deposit_receipt).unwrap();
    bad_account.set_data_from_slice(&data);
    ctx.set_account(&deposit_receipt_pda, &bad_account);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&depositor.pubkey()),
        &[&depositor],
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
async fn test_fail_invalid_vault_address() {
    let (mut ctx, _stake_pool_accounts, depositor, _deposit_receipt_pda, mut instructions) =
        setup_with_ix().await;
    instructions[1].accounts[2] = AccountMeta::new(Pubkey::new_unique(), false);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&depositor.pubkey()),
        &[&depositor],
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
async fn test_fail_fee_token_account_not_owned_by_fee_wallet() {
    let (mut ctx, stake_pool_accounts, depositor, _deposit_receipt_pda, mut instructions) =
        setup_with_ix().await;
    let bad_fee_token_account = create_token_account(
        &mut ctx,
        &depositor.pubkey(),
        &stake_pool_accounts.pool_mint,
    )
    .await;
    instructions[1].accounts[4] = AccountMeta::new(bad_fee_token_account, false);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&depositor.pubkey()),
        &[&depositor],
        ctx.last_blockhash,
    );

    assert_transaction_err(
        &mut ctx,
        tx,
        InstructionError::Custom(StakeDepositInterceptorError::InvalidFeeTokenAccount as u32),
    )
    .await;
}

#[tokio::test]
async fn test_fail_destination_token_account_not_owned_by_owner() {
    let (mut ctx, stake_pool_accounts, depositor, _deposit_receipt_pda, mut instructions) =
        setup_with_ix().await;
    let bad_owner = ctx.payer.pubkey();
    let bad_dest_token_account =
        create_token_account(&mut ctx, &bad_owner, &stake_pool_accounts.pool_mint).await;
    instructions[1].accounts[3] = AccountMeta::new(bad_dest_token_account, false);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&depositor.pubkey()),
        &[&depositor],
        ctx.last_blockhash,
    );

    assert_transaction_err(
        &mut ctx,
        tx,
        InstructionError::Custom(
            StakeDepositInterceptorError::InvalidDestinationTokenAccount as u32,
        ),
    )
    .await;
}
