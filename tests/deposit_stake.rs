mod helpers;

use helpers::{
    airdrop_lamports, create_stake_account, create_stake_deposit_authority, create_token_account,
    create_validator_and_add_to_pool, delegate_stake_account, get_account, get_account_data_deserialized,
    program_test_context_with_stake_pool_state, stake_pool_update_all,
};
use solana_sdk::{
    borsh1::try_from_slice_unchecked,
    native_token::LAMPORTS_PER_SOL,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    stake::{self, instruction::delegate_stake},
    system_instruction,
    transaction::Transaction,
};
use stake_deposit_interceptor::{
    instruction::derive_stake_pool_deposit_stake_authority, state::StakePoolDepositStakeAuthority,
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
    //  TODO update stake_pool's stake_deposit_authority
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
    let stake_amount = 2 * LAMPORTS_PER_SOL;
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
    let (deposit_stake_authority_pubkey, _bump) = derive_stake_pool_deposit_stake_authority(
        &stake_deposit_interceptor::id(),
        &stake_pool_accounts.stake_pool,
    );
    let deposit_stake_authority = get_account_data_deserialized::<StakePoolDepositStakeAuthority>(
        &mut ctx.banks_client,
        &deposit_stake_authority_pubkey,
    )
    .await;

    // signers: `deposit_stake_withdraw_authority`, `stake_pool_deposit_authority`
    // TODO who is `stake_pool_deposit_authority`?? It's our PDA deposit_stake_authority_pubkey

    // Actually test DepositStake
    let deposit_stake_instructions =
        stake_deposit_interceptor::instruction::create_deposit_stake_instruction(
            &stake_deposit_interceptor::id(),
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
        );

    let tx = Transaction::new_signed_with_payer(
        &deposit_stake_instructions,
        Some(&depositor.pubkey()),
        &[&depositor],
        ctx.last_blockhash,
    );

    ctx.banks_client.process_transaction(tx).await.unwrap();

    let vault_account = get_account(&mut ctx.banks_client, &deposit_stake_authority.vault).await;
    let vault = spl_token::state::Account::unpack(&vault_account.data).unwrap();

    // TODO be more prudent by checking the exact amount of tokens based on stake_amount.
    // assert LST was transfer to the vault
    assert!(vault.amount > 0);


    // TODO assert DepositReceipt has valid data
}

// TODO test incorrect TokenAccount for LST
