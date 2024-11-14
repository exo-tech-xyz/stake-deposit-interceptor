use solana_program_test::{processor, BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::AccountSharedData,
    instruction::InstructionError,
    program_error::ProgramError,
    pubkey::Pubkey,
    transaction::{Transaction, TransactionError},
    transport::TransportError,
};

use super::{create_stake_pool, StakePoolAccounts};

pub fn program_test_with_stake_pool_program() -> ProgramTest {
    let mut program_test = ProgramTest::default();
    program_test.add_program(
        "spl_stake_pool",
        spl_stake_pool::id(),
        processor!(spl_stake_pool::processor::Processor::process),
    );
    program_test.add_program(
        "stake_deposit_interceptor",
        stake_deposit_interceptor::id(),
        processor!(stake_deposit_interceptor::processor::Processor::process),
    );
    program_test
}

pub async fn program_test_context_with_stake_pool_state() -> (ProgramTestContext, StakePoolAccounts)
{
    let mut ctx = program_test_with_stake_pool_program()
        .start_with_context()
        .await;
    let stake_pool_accounts = create_stake_pool(&mut ctx).await;
    (ctx, stake_pool_accounts)
}

/// Clones all the existing account information and data to a new account. Returns the
/// new address of the account.
pub async fn clone_account_to_new_address(
    ctx: &mut ProgramTestContext,
    address: &Pubkey,
) -> Pubkey {
    let new_address = Pubkey::new_unique();
    let original = ctx
        .banks_client
        .get_account(*address)
        .await
        .unwrap()
        .unwrap();
    let mut bad_account =
        AccountSharedData::new(original.lamports, original.data.len(), &original.owner);
    bad_account.set_data_from_slice(&original.data);
    ctx.set_account(&new_address, &bad_account);
    new_address
}

/// Given a transaction and an error, assert the exact error is thrown during transaction
/// execution in the ProgramTestContext.
pub async fn assert_transaction_err(
    ctx: &mut ProgramTestContext,
    tx: Transaction,
    tx_error: InstructionError,
) {
    let transaction_error: TransportError = ctx
        .banks_client
        .process_transaction(tx)
        .await
        .err()
        .expect("Should have errored")
        .into();

    match transaction_error {
        TransportError::TransactionError(TransactionError::InstructionError(_, error)) => {
            assert_eq!(error, tx_error);
        }
        _ => panic!("Wrong error"),
    };
}
