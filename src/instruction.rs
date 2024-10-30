use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};
use spl_associated_token_account::get_associated_token_address;

/// Initialize arguments for StakePoolDepositStakeAuthority
#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct InitStakePoolDepositStakeAuthorityArgs {
    pub fee_wallet: Pubkey,
    pub cool_down_period: u64,
    pub initial_fee_rate: u32,
    pub bump_seed: u8,
}

/// Update arguments for StakePoolDepositStakeAuthority
#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct UpdateStakePoolDepositStakeAuthorityArgs {
    pub fee_wallet: Option<Pubkey>,
    pub cool_down_period: Option<u64>,
    pub initial_fee_rate: Option<u32>,
}

/// Instructions supported by the StakeDepositInterceptor program.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum StakeDepositInterceptorInstruction {
    ///   Initializes the StakePoolDepositStakeAuthority for the given StakePool.
    ///
    ///   0. `[w,s]` Payer that will fund the StakePoolDepositStakeAuthority account.
    ///   1. `[w]` New StakePoolDepositStakeAuthority to create.
    ///   2. `[w]` New ATA owned by the `StakePoolDepositStakeAuthority` to create.
    ///   3. `[s]` Authority
    ///   4. `[]` StakePool
    ///   5. `[]` StakePool's Pool Mint
    ///   6. `[]` StakePool Program ID
    ///   7. `[]` Token program
    ///   8. `[]` System program
    InitStakePoolDepositStakeAuthority(InitStakePoolDepositStakeAuthorityArgs),
    ///   Updates the StakePoolDepositStakeAuthority for the given StakePool.
    ///
    ///   0. `[w]` StakePoolDepositStakeAuthority PDA to be updated
    ///   1. `[s]` Authority
    UpdateStakePoolDepositStakeAuthority(UpdateStakePoolDepositStakeAuthorityArgs),
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
    stake_pool_mint: &Pubkey,
    stake_pool_manager: &Pubkey,
    stake_pool_program_id: &Pubkey,
    token_program_id: &Pubkey,
    fee_wallet: &Pubkey,
    cool_down_period: u64,
    initial_fee_rate: u32,
    authority: &Pubkey,
) -> Instruction {
    let (deposit_stake_authority_pubkey, bump_seed) =
        derive_stake_pool_deposit_stake_authority(program_id, stake_pool);
    let vault_ata = get_associated_token_address(&deposit_stake_authority_pubkey, stake_pool_mint);
    let args = InitStakePoolDepositStakeAuthorityArgs {
        fee_wallet: *fee_wallet,
        initial_fee_rate,
        cool_down_period,
        bump_seed,
    };
    let accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new(deposit_stake_authority_pubkey, false),
        AccountMeta::new(vault_ata, false),
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new_readonly(*stake_pool, false),
        AccountMeta::new_readonly(*stake_pool_mint, false),
        AccountMeta::new_readonly(*stake_pool_manager, true),
        AccountMeta::new_readonly(*stake_pool_program_id, false),
        AccountMeta::new_readonly(*token_program_id, false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(
            &StakeDepositInterceptorInstruction::InitStakePoolDepositStakeAuthority(args),
        )
        .unwrap(),
    }
}

pub fn create_update_deposit_stake_authority_instruction(
    program_id: &Pubkey,
    stake_pool: &Pubkey,
    authority: &Pubkey,
    new_authority: Option<Pubkey>,
    fee_wallet: Option<Pubkey>,
    cool_down_period: Option<u64>,
    initial_fee_rate: Option<u32>,
) -> Instruction {
    let (deposit_stake_authority_pubkey, _bump_seed) =
        derive_stake_pool_deposit_stake_authority(program_id, stake_pool);
    let args = UpdateStakePoolDepositStakeAuthorityArgs {
        fee_wallet: fee_wallet,
        initial_fee_rate: initial_fee_rate,
        cool_down_period: cool_down_period,
    };
    let mut accounts = vec![
        AccountMeta::new(deposit_stake_authority_pubkey, false),
        AccountMeta::new_readonly(*authority, true),
    ];
    if let Some(new_authority) = new_authority {
        accounts.push(AccountMeta::new(new_authority, true));
    }
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(
            &StakeDepositInterceptorInstruction::UpdateStakePoolDepositStakeAuthority(args),
        )
        .unwrap(),
    }
}
