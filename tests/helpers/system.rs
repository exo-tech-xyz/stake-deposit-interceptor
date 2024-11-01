use solana_program_test::{BanksClient, ProgramTestContext};
use solana_sdk::{
    account::Account as SolanaAccount, pubkey::Pubkey, signer::Signer, system_instruction,
    transaction::Transaction,
};

/// Airdrop tokens from the `ProgramTestContext` payer to a designated Pubkey.
pub async fn airdrop_lamports(ctx: &mut ProgramTestContext, receiver: &Pubkey, amount: u64) {
    ctx.banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[system_instruction::transfer(
                &ctx.payer.pubkey(),
                &receiver,
                amount,
            )],
            Some(&ctx.payer.pubkey()),
            &[&ctx.payer],
            ctx.last_blockhash,
        ))
        .await
        .unwrap();
}

/// Fetch an Account from ProgramTestContext.
pub async fn get_account(banks_client: &mut BanksClient, pubkey: &Pubkey) -> SolanaAccount {
    banks_client
        .get_account(*pubkey)
        .await
        .expect("client error")
        .expect("account not found")
}
