mod helpers;

use helpers::program_test_context_with_stake_pool_state;
use solana_sdk::{
    borsh1::try_from_slice_unchecked, program_pack::Pack, signature::Keypair, signer::Signer,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use stake_deposit_interceptor::{
    instruction::derive_stake_pool_deposit_stake_authority, state::StakePoolDepositStakeAuthority,
};

#[tokio::test]
async fn test_init_deposit_stake_authority() {
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

    let fee_wallet = Keypair::new();
    let authority = Keypair::new();
    let cool_down_period = 100;
    let initial_fee_rate = 20;
    let init_ix =
        stake_deposit_interceptor::instruction::create_init_deposit_stake_authority_instruction(
            &stake_deposit_interceptor::id(),
            &ctx.payer.pubkey(),
            &stake_pool_accounts.stake_pool,
            &stake_pool.pool_mint,
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
    let vault_ata =
        get_associated_token_address(&deposit_stake_authority_pubkey, &stake_pool.pool_mint);

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

    let deposit_stake_authority: StakePoolDepositStakeAuthority =
        try_from_slice_unchecked(&account.data.as_slice()).unwrap();
    let vault_token_account = spl_token::state::Account::unpack(vault_account.data.as_slice()).unwrap();
    assert_eq!(vault_token_account.mint, stake_pool.pool_mint);
    assert_eq!(vault_token_account.amount, 0);
    assert_eq!(vault_token_account.owner, deposit_stake_authority_pubkey);

    assert_eq!(deposit_stake_authority.authority, authority.pubkey());
    let actual_cool_down_period: u64 = deposit_stake_authority.cool_down_period.into();
    let actual_initial_fee_rate: u32 = deposit_stake_authority.inital_fee_rate.into();
    assert_eq!(actual_cool_down_period, cool_down_period);
    assert_eq!(actual_initial_fee_rate, initial_fee_rate);
    assert_eq!(deposit_stake_authority.stake_pool, stake_pool_accounts.stake_pool);
    assert_eq!(deposit_stake_authority.pool_mint, stake_pool.pool_mint);
    assert_eq!(
        deposit_stake_authority.stake_pool_program_id,
        spl_stake_pool::id()
    );
    assert_eq!(deposit_stake_authority.fee_wallet, fee_wallet.pubkey());
    assert_eq!(deposit_stake_authority.vault, vault_ata);
}

// TODO add tests for validations
