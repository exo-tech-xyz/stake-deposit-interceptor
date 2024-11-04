use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    stake, system_program, sysvar,
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

/// Arguments for DepositStake.
///
/// NOTE: we must pass the owner as a separate arg (or account) as
/// by the time the DepositStake instruction is processed, the
/// authorized staker & withdrawer has become a PDA owned by this
/// program and not the original authorized pubkey for the Stake Account.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct DepositStakeArgs {
    pub base: Pubkey,
    /// The pubkey that will own the DepositReceipt and thus
    /// be able to claim the minted LST.
    pub owner: Pubkey,
}

/// Arguments for DepositStakeWithSlippage.
///
/// NOTE: we must pass the owner as a separate arg (or account) as
/// by the time the DepositStake instruction is processed, the
/// authorized staker & withdrawer has become a PDA owned by this
/// program and not the original authorized pubkey for the Stake Account.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct DepositStakeWithSlippageArgs {
    pub base: Pubkey,
    /// The pubkey that will own the DepositReceipt and thus
    /// be able to claim the minted LST.
    pub owner: Pubkey,
    /// Slippage amount as defined in SPL stake-pool program.
    pub minimum_pool_tokens_out: u64,
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
    // TODO fix all the numbering
    ///   Deposit some stake into the pool. The output is a "pool" token
    ///   representing ownership into the pool. Inputs are converted to the
    ///   current ratio.
    ///
    ///   0. `[w]` payer of the new account rent
    ///   0. `[]` stake pool program id
    ///   0. `[w]` DepositReceipt to be created
    ///   0. `[w]` Stake pool
    ///   1. `[w]` Validator stake list storage account
    ///   2. `[s]` Stake pool deposit authority (aka the StakePoolDepositStakeAuthority PDA)
    ///   3. `[]` Stake pool withdraw authority
    ///   4. `[w]` Stake account to join the pool (withdraw authority for the
    ///      stake account should be first set to the stake pool deposit
    ///      authority)
    ///   5. `[w]` Validator stake account for the stake account to be merged
    ///      with
    ///   6. `[w]` Reserve stake account, to withdraw rent exempt reserve
    ///   7. `[w]` User account to receive pool tokens
    ///   8. `[w]` Account to receive pool fee tokens
    ///   9. `[w]` Account to receive a portion of pool fee tokens as referral
    ///      fees
    ///   10. `[w]` Pool token mint account
    ///   11. '[]' Sysvar clock account
    ///   12. '[]' Sysvar stake history account
    ///   13. `[]` Pool token program id,
    ///   14. `[]` Stake program id,
    ///   14. `[]` System program id,
    DepositStake(DepositStakeArgs),
    // TODO fix account numbering
    ///   Deposit some stake into the pool, with a specified slippage
    ///   constraint. The output is a "pool" token representing ownership
    ///   into the pool. Inputs are converted at the current ratio.
    ///
    ///   0. `[]` stake pool program id
    ///   0. `[w]` Stake pool
    ///   1. `[w]` Validator stake list storage account
    ///   2. `[s]` Stake pool deposit authority (aka the StakePoolDepositStakeAuthority PDA)
    ///   3. `[]` Stake pool withdraw authority
    ///   4. `[w]` Stake account to join the pool (withdraw authority for the
    ///      stake account should be first set to the stake pool deposit
    ///      authority)
    ///   5. `[w]` Validator stake account for the stake account to be merged
    ///      with
    ///   6. `[w]` Reserve stake account, to withdraw rent exempt reserve
    ///   7. `[w]` User account to receive pool tokens
    ///   8. `[w]` Account to receive pool fee tokens
    ///   9. `[w]` Account to receive a portion of pool fee tokens as referral
    ///      fees
    ///   10. `[w]` Pool token mint account
    ///   11. '[]' Sysvar clock account
    ///   12. '[]' Sysvar stake history account
    ///   13. `[]` Pool token program id,
    ///   14. `[]` Stake program id,
    DepositStakeWithSlippage(DepositStakeWithSlippageArgs),
    // TODO DepositStakeWithSlippage
}

pub const STAKE_POOL_DEPOSIT_STAKE_AUTHORITY: &[u8] = b"deposit_stake_authority";
pub const DEPOSIT_RECEIPT: &[u8] = b"deposit_receipt";

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

/// Derive the DepositReceipt pubkey for a given program
pub fn derive_stake_deposit_receipt(
    program_id: &Pubkey,
    owner: &Pubkey,
    stake_pool: &Pubkey,
    base: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            DEPOSIT_RECEIPT,
            &owner.to_bytes(),
            &stake_pool.to_bytes(),
            &base.to_bytes(),
        ],
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

fn deposit_stake_internal(
    program_id: &Pubkey,
    payer: &Pubkey,
    stake_pool_program_id: &Pubkey,
    stake_pool: &Pubkey,
    validator_list_storage: &Pubkey,
    stake_pool_deposit_authority: &Pubkey,
    stake_pool_withdraw_authority: &Pubkey,
    deposit_stake_address: &Pubkey,
    deposit_stake_withdraw_authority: &Pubkey,
    validator_stake_account: &Pubkey,
    reserve_stake_account: &Pubkey,
    pool_tokens_to: &Pubkey,
    manager_fee_account: &Pubkey,
    referrer_pool_tokens_account: &Pubkey,
    pool_mint: &Pubkey,
    token_program_id: &Pubkey,
    base: &Pubkey,
    minimum_pool_tokens_out: Option<u64>,
) -> Vec<Instruction> {
    let (deposit_receipt_pubkey, _bump_seed) = derive_stake_deposit_receipt(
        program_id,
        deposit_stake_withdraw_authority,
        stake_pool,
        base,
    );
    let mut instructions = vec![];
    let mut accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(*stake_pool_program_id, false),
        AccountMeta::new(deposit_receipt_pubkey, false),
        AccountMeta::new(*stake_pool, false),
        AccountMeta::new(*validator_list_storage, false),
        // This is our PDA that will signed the CPI
        AccountMeta::new_readonly(*stake_pool_deposit_authority, false),
    ];
    // NOTE: Assumes the withdrawer and staker authorities are the same (i.e. `deposit_stake_withdraw_authority`).
    instructions.extend_from_slice(&[
        stake::instruction::authorize(
            deposit_stake_address,
            deposit_stake_withdraw_authority,
            stake_pool_deposit_authority,
            stake::state::StakeAuthorize::Staker,
            None,
        ),
        stake::instruction::authorize(
            deposit_stake_address,
            deposit_stake_withdraw_authority,
            stake_pool_deposit_authority,
            stake::state::StakeAuthorize::Withdrawer,
            None,
        ),
    ]);

    accounts.extend_from_slice(&[
        AccountMeta::new_readonly(*stake_pool_withdraw_authority, false),
        AccountMeta::new(*deposit_stake_address, false),
        AccountMeta::new(*validator_stake_account, false),
        AccountMeta::new(*reserve_stake_account, false),
        AccountMeta::new(*pool_tokens_to, false),
        AccountMeta::new(*manager_fee_account, false),
        AccountMeta::new(*referrer_pool_tokens_account, false),
        AccountMeta::new(*pool_mint, false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
        AccountMeta::new_readonly(sysvar::stake_history::id(), false),
        AccountMeta::new_readonly(*token_program_id, false),
        AccountMeta::new_readonly(stake::program::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ]);
    instructions.push(
        if let Some(minimum_pool_tokens_out) = minimum_pool_tokens_out {
            let args = DepositStakeWithSlippageArgs {
                base: *base,
                owner: *deposit_stake_withdraw_authority,
                minimum_pool_tokens_out,
            };
            Instruction {
                program_id: *program_id,
                accounts,
                data: borsh::to_vec(
                    &StakeDepositInterceptorInstruction::DepositStakeWithSlippage(args),
                )
                .unwrap(),
            }
        } else {
            let args = DepositStakeArgs {
                base: *base,
                owner: *deposit_stake_withdraw_authority,
            };
            Instruction {
                program_id: *program_id,
                accounts,
                data: borsh::to_vec(&StakeDepositInterceptorInstruction::DepositStake(args))
                    .unwrap(),
            }
        },
    );
    instructions
}

/// Creates instructions required to deposit into a stake pool, given a stake
/// account owned by the user.
pub fn create_deposit_stake_instruction(
    program_id: &Pubkey,
    payer: &Pubkey,
    stake_pool_program_id: &Pubkey,
    stake_pool: &Pubkey,
    validator_list_storage: &Pubkey,
    stake_pool_withdraw_authority: &Pubkey,
    deposit_stake_address: &Pubkey,
    deposit_stake_withdraw_authority: &Pubkey,
    validator_stake_account: &Pubkey,
    reserve_stake_account: &Pubkey,
    pool_tokens_to: &Pubkey,
    manager_fee_account: &Pubkey,
    referrer_pool_tokens_account: &Pubkey,
    pool_mint: &Pubkey,
    token_program_id: &Pubkey,
    base: &Pubkey,
) -> Vec<Instruction> {
    // The StakePool's deposit authority is assumed to be the PDA owned by
    // the stake-deposit-interceptor program
    let (deposit_stake_authority_pubkey, _bump_seed) =
        derive_stake_pool_deposit_stake_authority(program_id, stake_pool);
    deposit_stake_internal(
        program_id,
        payer,
        stake_pool_program_id,
        stake_pool,
        validator_list_storage,
        &deposit_stake_authority_pubkey,
        stake_pool_withdraw_authority,
        deposit_stake_address,
        deposit_stake_withdraw_authority,
        validator_stake_account,
        reserve_stake_account,
        pool_tokens_to,
        manager_fee_account,
        referrer_pool_tokens_account,
        pool_mint,
        token_program_id,
        base,
        None,
    )
}

/// Creates instructions required to deposit into a stake pool, given a stake
/// account owned by the user. StakePool program verifies the minimum tokens are minted.
pub fn create_deposit_stake_with_slippage_nstruction(
    program_id: &Pubkey,
    payer: &Pubkey,
    stake_pool_program_id: &Pubkey,
    stake_pool: &Pubkey,
    validator_list_storage: &Pubkey,
    stake_pool_withdraw_authority: &Pubkey,
    deposit_stake_address: &Pubkey,
    deposit_stake_withdraw_authority: &Pubkey,
    validator_stake_account: &Pubkey,
    reserve_stake_account: &Pubkey,
    pool_tokens_to: &Pubkey,
    manager_fee_account: &Pubkey,
    referrer_pool_tokens_account: &Pubkey,
    pool_mint: &Pubkey,
    token_program_id: &Pubkey,
    base: &Pubkey,
    minimum_pool_tokens_out: u64,
) -> Vec<Instruction> {
    // The StakePool's deposit authority is assumed to be the PDA owned by
    // the stake-deposit-interceptor program
    let (deposit_stake_authority_pubkey, _bump_seed) =
        derive_stake_pool_deposit_stake_authority(program_id, stake_pool);
    deposit_stake_internal(
        program_id,
        payer,
        stake_pool_program_id,
        stake_pool,
        validator_list_storage,
        &deposit_stake_authority_pubkey,
        stake_pool_withdraw_authority,
        deposit_stake_address,
        deposit_stake_withdraw_authority,
        validator_stake_account,
        reserve_stake_account,
        pool_tokens_to,
        manager_fee_account,
        referrer_pool_tokens_account,
        pool_mint,
        token_program_id,
        base,
        Some(minimum_pool_tokens_out),
    )
}

