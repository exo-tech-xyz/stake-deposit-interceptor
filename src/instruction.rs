use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

/// Initialize arguments for StakePoolDepositStakeAuthority
#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct InitStakePoolDepositStakeAuthorityArgs {
    pub fee_wallet: Pubkey,
    pub cool_down_period: u64,
    pub initial_fee_rate: u32,
    pub bump_seed: u8,
}

/// Instructions supported by the StakeDepositInterceptor program.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum StakeDepositInterceptorInstruction {
    ///   Initializes the StakePoolDepositStakeAuthority for the program.
    ///
    ///   0. `[w,s]` Payer that will fund the StakePoolDepositStakeAuthority account.
    ///   1. `[w]` New StakePoolDepositStakeAuthority to create.
    ///   2. `[s]` Authority
    ///   3. `[]` StakePool
    ///   4. `[]` StakePool Program ID
    ///   5. `[]` System program
    StakePoolDepositStakeAuthority(InitStakePoolDepositStakeAuthorityArgs),
    DepositStake,
}

pub const STAKE_POOL_DEPOSIT_STAKE_AUTHORITY: &[u8] = b"deposit_stake_authority";

/// Derive the StakePoolDepositStakeAuthority pubkey for a given program
pub fn derive_stake_pool_deposit_stake_authority(
    program_id: &Pubkey,
    stake_pool: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[STAKE_POOL_DEPOSIT_STAKE_AUTHORITY, &stake_pool.to_bytes()],
        program_id,
    )
}

/// Creates instruction to set up the StakePoolDepositStakeAuthority to be used in the
pub fn create_init_deposit_stake_authority_instruction(
    program_id: &Pubkey,
    payer: &Pubkey,
    stake_pool: &Pubkey,
    stake_pool_manager: &Pubkey,
    stake_pool_program_id: &Pubkey,
    fee_wallet: &Pubkey,
    cool_down_period: u64,
    initial_fee_rate: u32,
    authority: &Pubkey,
) -> Instruction {
    let (deposit_stake_authority_pubkey, bump_seed) =
        derive_stake_pool_deposit_stake_authority(program_id, stake_pool);
    let args = InitStakePoolDepositStakeAuthorityArgs {
        fee_wallet: *fee_wallet,
        initial_fee_rate,
        cool_down_period,
        bump_seed,
    };
    let accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new(deposit_stake_authority_pubkey, false),
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new_readonly(*stake_pool, false),
        AccountMeta::new_readonly(*stake_pool_manager, true),
        AccountMeta::new_readonly(*stake_pool_program_id, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(
            &StakeDepositInterceptorInstruction::StakePoolDepositStakeAuthority(args),
        )
        .unwrap(),
    }
}
