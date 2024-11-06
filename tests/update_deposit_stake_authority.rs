mod helpers;

use helpers::{create_stake_deposit_authority, program_test_context_with_stake_pool_state};
use solana_sdk::{
    borsh1::try_from_slice_unchecked, signature::Keypair, signer::Signer, transaction::Transaction,
};
use stake_deposit_interceptor::{
    instruction::derive_stake_pool_deposit_stake_authority, state::StakePoolDepositStakeAuthority,
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

    let deposit_stake_authority: StakePoolDepositStakeAuthority =
        try_from_slice_unchecked(&account.data.as_slice()).unwrap();

    let actual_cool_down_period: u64 = deposit_stake_authority.cool_down_period.into();
    let actual_initial_fee_rate: u32 = deposit_stake_authority.inital_fee_rate.into();
    assert_eq!(actual_cool_down_period, cool_down_period);
    assert_eq!(actual_initial_fee_rate, initial_fee_rate);
    assert_eq!(deposit_stake_authority.fee_wallet, fee_wallet.pubkey());
    assert_eq!(deposit_stake_authority.authority, new_authority.pubkey());
}
