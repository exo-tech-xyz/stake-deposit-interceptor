use borsh::BorshDeserialize;
use solana_program_test::{BanksClient, ProgramTestContext};
use solana_sdk::{
    account::Account as SolanaAccount, borsh1::try_from_slice_unchecked, pubkey::Pubkey, signer::Signer, system_instruction, transaction::Transaction
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

/// Fetch an account and deserialize based on type.
pub async fn get_account_data_deserialized<T: BorshDeserialize>(banks_client: &mut BanksClient, pubkey: &Pubkey) -> T {
    let account = get_account(banks_client, pubkey).await;
    try_from_slice_unchecked::<T>(&account.data.as_slice()).unwrap()
}
