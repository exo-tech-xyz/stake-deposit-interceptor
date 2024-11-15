use solana_program::pubkey::Pubkey;
use solana_program_test::ProgramTestContext;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

/// Create and initialize a `StakePoolDepositStakeAuthority`.
pub async fn create_stake_deposit_authority(
    ctx: &mut ProgramTestContext,
    stake_pool_pubkey: &Pubkey,
    stake_pool_mint: &Pubkey,
    authority: &Keypair,
    fee_wallet_address: Option<&Pubkey>,
) {
    let mut fee_wallet = Pubkey::new_unique();
    if let Some(fee_wallet_address) = fee_wallet_address {
        fee_wallet = *fee_wallet_address;
    }
    let cool_down_seconds = 100;
    let initial_fee_bps = 20;
    let init_ix =
        stake_deposit_interceptor::instruction::create_init_deposit_stake_authority_instruction(
            &stake_deposit_interceptor::id(),
            &ctx.payer.pubkey(),
            &stake_pool_pubkey,
            stake_pool_mint,
            &spl_stake_pool::id(),
            &spl_token::id(),
            &fee_wallet,
            cool_down_seconds,
            initial_fee_bps,
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
