mod helpers;

use helpers::{
    airdrop_lamports, create_stake_account, create_stake_deposit_authority, create_token_account,
    create_validator_and_add_to_pool, delegate_stake_account, get_account,
    get_account_data_deserialized, program_test_context_with_stake_pool_state,
    stake_pool_update_all, update_stake_deposit_authority, StakePoolAccounts,
    ValidatorStakeAccount,
};
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    borsh1::try_from_slice_unchecked,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    stake::{self},
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use stake_deposit_interceptor::{
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
async fn claim_pool_tokens_success() {
    let (
        mut ctx,
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
    );

    let tx = Transaction::new_signed_with_payer(
        &[create_fee_token_account_ix, ix],
        Some(&depositor.pubkey()),
        &[&depositor],
        ctx.last_blockhash,
    );

    ctx.banks_client.process_transaction(tx).await.unwrap();

    // TODO asset Destination token account should have received pool tokens

    // TODO assert Fees should have been paid

    // DepositReceipt account should have been closed
    let deposit_receipt_account = ctx
        .banks_client
        .get_account(deposit_receipt_pda)
        .await
        .unwrap();
    assert!(deposit_receipt_account.is_none());
}

// TODO test fee token account not owned by fee wallet
// TODO test incorrect vault token account
// TODO test incorrect StakePoolDepositAuthority account
// TODO test destination account not owned by DepositReceipt owner
