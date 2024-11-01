mod helpers;

use helpers::{
    airdrop_lamports, create_stake_account, create_stake_deposit_authority, create_token_account,
    create_validator_and_add_to_pool, delegate_stake_account,
    program_test_context_with_stake_pool_state, stake_pool_update_all,
};
use solana_sdk::{
    borsh1::try_from_slice_unchecked,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    stake::{self, instruction::delegate_stake},
    system_instruction,
    transaction::Transaction,
};

#[tokio::test]
async fn test_deposit_stake() {
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
    // Add a validator to the stake_pool
    let validator_stake_accounts =
        create_validator_and_add_to_pool(&mut ctx, &stake_pool_accounts).await;

    let authority = Keypair::new();
    create_stake_deposit_authority(
        &mut ctx,
        &stake_pool_accounts.stake_pool,
        &stake_pool.pool_mint,
        &authority,
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
    let depositor_stake_account = create_stake_account(
        &mut ctx.banks_client,
        &depositor,
        &authorized,
        &lockup,
        2 * LAMPORTS_PER_SOL,
        ctx.last_blockhash,
    )
    .await;

    // Create a TokenAccount for the "Depositor" of the StakePool's `pool_mint`.
    let depositor_lst_account = create_token_account(
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

    stake_pool_update_all(
        &mut ctx.banks_client,
        &ctx.payer,
        &stake_pool_accounts,
        &ctx.last_blockhash,
        false,
    ).await;

    

    // TODO assert LST is held by vault
    // TODO assert DepositReceipt has valid data
}
