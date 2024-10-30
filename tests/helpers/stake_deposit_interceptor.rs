use solana_program::pubkey::Pubkey;
use solana_program_test::ProgramTestContext;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

/// Create and initialize a `StakePoolDepositStakeAuthority`.
pub async fn create_stake_deposit_authority(
    ctx: &mut ProgramTestContext,
    stake_pool_pubkey: &Pubkey,
    authority: &Keypair,
) {
    let fee_wallet = Keypair::new();
    let cool_down_period = 100;
    let initial_fee_rate = 20;
    let init_ix =
        stake_deposit_interceptor::instruction::create_init_deposit_stake_authority_instruction(
            &stake_deposit_interceptor::id(),
            &ctx.payer.pubkey(),
            &stake_pool_pubkey,
            &ctx.payer.pubkey(),
            &spl_stake_pool::id(),
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
}
