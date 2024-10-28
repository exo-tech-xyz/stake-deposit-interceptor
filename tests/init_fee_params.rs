mod helpers;

use helpers::program_test_context_with_stake_pool_state;
use solana_sdk::{
    borsh1::try_from_slice_unchecked, signature::Keypair, signer::Signer, transaction::Transaction,
};
use stake_deposit_interceptor::{instruction::derive_fee_parameters, state::FeeParameters};

#[tokio::test]
async fn test_init_fee_params() {
    let mut ctx = program_test_context_with_stake_pool_state().await;

    let authority = Keypair::new();
    let cool_down_period = 100;
    let initial_fee_rate = 20;
    let init_fee_params_ix =
        stake_deposit_interceptor::instruction::create_init_fee_parameters_instruction(
            &stake_deposit_interceptor::id(),
            &ctx.payer.pubkey(),
            cool_down_period,
            initial_fee_rate,
            &authority.pubkey(),
        );

    let tx = Transaction::new_signed_with_payer(
        &[init_fee_params_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &authority],
        ctx.last_blockhash,
    );

    ctx.banks_client.process_transaction(tx).await.unwrap();

    let (fee_parameter_pubkey, _bump_seed) = derive_fee_parameters(&stake_deposit_interceptor::ID);

    let fee_parameters_account = ctx
        .banks_client
        .get_account(fee_parameter_pubkey)
        .await
        .unwrap()
        .unwrap();
    let fee_parameters: FeeParameters =
        try_from_slice_unchecked(&fee_parameters_account.data.as_slice()).unwrap();
      
    assert_eq!(fee_parameters.authority, authority.pubkey());
    assert_eq!(fee_parameters.cool_down_period, cool_down_period);
    assert_eq!(fee_parameters.inital_fee_rate, initial_fee_rate);

    assert!(true);
}
