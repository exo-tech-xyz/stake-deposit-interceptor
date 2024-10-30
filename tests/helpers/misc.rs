use solana_program_test::{processor, ProgramTest, ProgramTestContext};
use solana_program::pubkey::Pubkey;

use super::create_stake_pool;

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

pub async fn program_test_context_with_stake_pool_state() -> (ProgramTestContext, Pubkey) {
    let mut ctx = program_test_with_stake_pool_program()
        .start_with_context()
        .await;
    let stake_pool = create_stake_pool(&mut ctx).await;
    (ctx, stake_pool)
}