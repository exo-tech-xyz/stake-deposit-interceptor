use solana_program_test::ProgramTestContext;
use solana_sdk::clock::Clock;

/// Set the sysvar Clock's time to the given timestamp.
pub async fn set_clock_time(ctx: &mut ProgramTestContext, time_in_seconds: i64) {
  let clock: Clock = ctx.banks_client.get_sysvar().await.unwrap();

  let mut updated_clock = clock.clone();
  updated_clock.unix_timestamp = time_in_seconds;

  ctx.set_sysvar(&updated_clock);
}
