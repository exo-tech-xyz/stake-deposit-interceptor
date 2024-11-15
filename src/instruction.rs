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
    pub cool_down_seconds: u64,
    pub initial_fee_bps: u32,
    pub base: Pubkey,
}

/// Update arguments for StakePoolDepositStakeAuthority
#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct UpdateStakePoolDepositStakeAuthorityArgs {
    pub fee_wallet: Option<Pubkey>,
    pub cool_down_seconds: Option<u64>,
    pub initial_fee_bps: Option<u32>,
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
    ///   8. `[]` Associated Token program
    ///   9. `[]` System program
    InitStakePoolDepositStakeAuthority(InitStakePoolDepositStakeAuthorityArgs),
    ///   Updates the StakePoolDepositStakeAuthority for the given StakePool.
    ///
    ///   0. `[w]` StakePoolDepositStakeAuthority PDA to be updated
    ///   1. `[s]` Authority
    ///   2. `[s]` (Optional) New authority
    UpdateStakePoolDepositStakeAuthority(UpdateStakePoolDepositStakeAuthorityArgs),
    ///   Deposit some stake into the pool. The "pool" token minted is held by the DepositReceipt's
    ///   Vault token Account rather than a token Account designated by the depositor.
    ///   Inputs are converted to the current ratio.
    ///
    ///   0. `[w]` payer of the new account rent
    ///   1. `[]` stake pool program id
    ///   2. `[w]` DepositReceipt to be created
    ///   3. `[w]` Stake pool
    ///   4. `[w]` Validator stake list storage account
    ///   5. `[s]` Stake pool deposit authority (aka the StakePoolDepositStakeAuthority PDA)
    ///   6. `[]` Stake pool withdraw authority
    ///   7. `[w]` Stake account to join the pool (withdraw authority for the
    ///      stake account should be first set to the stake pool deposit
    ///      authority)
    ///   8. `[w]` Validator stake account for the stake account to be merged
    ///      with
    ///   9. `[w]` Reserve stake account, to withdraw rent exempt reserve
    ///   10. `[w]` Vault account to receive pool tokens
    ///   11. `[w]` Account to receive pool fee tokens
    ///   12. `[w]` Account to receive a portion of pool fee tokens as referral
    ///      fees
    ///   13. `[w]` Pool token mint account
    ///   14. '[]' Sysvar clock account
    ///   15. '[]' Sysvar stake history account
    ///   16. `[]` Pool token program id,
    ///   17. `[]` Stake program id,
    ///   18. `[]` System program id,
    DepositStake(DepositStakeArgs),
    ///   Deposit some stake into the pool, with a specified slippage
    ///   constraint. The "pool" token minted is held by the DepositReceipt's
    ///   Vault token Account rather than a token Account designated by the depositor.
    ///   Inputs are converted to the current ratio.
    ///
    ///   0. `[w]` payer of the new account rent
    ///   1. `[]` stake pool program id
    ///   2. `[w]` DepositReceipt to be created
    ///   3. `[w]` Stake pool
    ///   4. `[w]` Validator stake list storage account
    ///   5. `[s]` Stake pool deposit authority (aka the StakePoolDepositStakeAuthority PDA)
    ///   6. `[]` Stake pool withdraw authority
    ///   7. `[w]` Stake account to join the pool (withdraw authority for the
    ///      stake account should be first set to the stake pool deposit
    ///      authority)
    ///   8. `[w]` Validator stake account for the stake account to be merged
    ///      with
    ///   9. `[w]` Reserve stake account, to withdraw rent exempt reserve
    ///   10. `[w]` Vault account to receive pool tokens
    ///   11. `[w]` Account to receive pool fee tokens
    ///   12. `[w]` Account to receive a portion of pool fee tokens as referral
    ///      fees
    ///   13. `[w]` Pool token mint account
    ///   14. '[]' Sysvar clock account
    ///   15. '[]' Sysvar stake history account
    ///   16. `[]` Pool token program id,
    ///   17. `[]` Stake program id,
    ///   18. `[]` System program id,
    DepositStakeWithSlippage(DepositStakeWithSlippageArgs),
    ///   Update the `owner` of the DepositReceipt so the new owner
    ///   has the authority to claim the "pool" tokens.
    ///
    ///   0. `[w]` DepositReceipt PDA
    ///   1. `[s]` current owner of the DepositReceipt
    ///   2. `[]` new owner for the DepositReceipt
    ChangeDepositReceiptOwner,
    ///   Claim the "pool" tokens held by the program from a former DepositStake
    ///   transaction. Fees will be deducted from the destination token account
    ///   if this instruction is invoked during the cool down period.
    ///
    ///   0. `[w]` DepositReceipt PDA
    ///   1. `[w,s]` owner of the DepositReceipt
    ///   2. `[w]` vault token account to send tokens from
    ///   3. `[w]` destination token account
    ///   4. `[w]` fee wallet token account
    ///   5. `[]` StakePoolDepositStakeAuthority PDA
    ///   6. `[]` Pool token mint
    ///   7. `[]` Token program id
    ///   8. `[]` System program id
    ClaimPoolTokens,
}

pub const STAKE_POOL_DEPOSIT_STAKE_AUTHORITY: &[u8] = b"deposit_stake_authority";
pub const DEPOSIT_RECEIPT: &[u8] = b"deposit_receipt";

/// Derive the StakePoolDepositStakeAuthority pubkey for a given program
pub fn derive_stake_pool_deposit_stake_authority(
    program_id: &Pubkey,
    stake_pool: &Pubkey,
    base: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            STAKE_POOL_DEPOSIT_STAKE_AUTHORITY,
            &stake_pool.to_bytes(),
            &base.to_bytes(),
        ],
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
    stake_pool_program_id: &Pubkey,
    token_program_id: &Pubkey,
    fee_wallet: &Pubkey,
    cool_down_seconds: u64,
    initial_fee_bps: u32,
    authority: &Pubkey,
    base: &Pubkey,
) -> Instruction {
    let (deposit_stake_authority_pubkey, _bump_seed) =
        derive_stake_pool_deposit_stake_authority(program_id, stake_pool, base);
    let vault_ata = get_associated_token_address(&deposit_stake_authority_pubkey, stake_pool_mint);
    let args = InitStakePoolDepositStakeAuthorityArgs {
        fee_wallet: *fee_wallet,
        initial_fee_bps,
        cool_down_seconds,
        base: *base,
    };
    let accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new(deposit_stake_authority_pubkey, false),
        AccountMeta::new(vault_ata, false),
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new_readonly(*stake_pool, false),
        AccountMeta::new_readonly(*stake_pool_mint, false),
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
    base: &Pubkey,
    new_authority: Option<Pubkey>,
    fee_wallet: Option<Pubkey>,
    cool_down_seconds: Option<u64>,
    initial_fee_bps: Option<u32>,
) -> Instruction {
    let (deposit_stake_authority_pubkey, _bump_seed) =
        derive_stake_pool_deposit_stake_authority(program_id, stake_pool, base);
    let args = UpdateStakePoolDepositStakeAuthorityArgs {
        fee_wallet: fee_wallet,
        initial_fee_bps,
        cool_down_seconds,
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
    deposit_receipt_base: &Pubkey,
    deposit_authority_base: &Pubkey,
) -> Vec<Instruction> {
    // The StakePool's deposit authority is assumed to be the PDA owned by
    // the stake-deposit-interceptor program
    let (deposit_stake_authority_pubkey, _bump_seed) =
        derive_stake_pool_deposit_stake_authority(program_id, stake_pool, deposit_authority_base);
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
        deposit_receipt_base,
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
    deposit_receipt_base: &Pubkey,
    deposit_authority_base: &Pubkey,
    minimum_pool_tokens_out: u64,
) -> Vec<Instruction> {
    // The StakePool's deposit authority is assumed to be the PDA owned by
    // the stake-deposit-interceptor program
    let (deposit_stake_authority_pubkey, _bump_seed) =
        derive_stake_pool_deposit_stake_authority(program_id, stake_pool, deposit_authority_base);
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
        deposit_receipt_base,
        Some(minimum_pool_tokens_out),
    )
}

/// Creates the Instruction to change the current owner of the DepositReceipt.
pub fn create_change_deposit_receipt_owner(
    program_id: &Pubkey,
    deposit_receipt_address: &Pubkey,
    owner: &Pubkey,
    new_owner: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*deposit_receipt_address, false),
        AccountMeta::new_readonly(*owner, true),
        AccountMeta::new_readonly(*new_owner, false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&StakeDepositInterceptorInstruction::ChangeDepositReceiptOwner)
            .unwrap(),
    }
}

/// Creates a ClaimPoolTokens instruction to transfer the held "pool" tokens to
/// destination token account. Also closes the DepositReceipt and refunds the owner.
pub fn create_claim_pool_tokens_instruction(
    program_id: &Pubkey,
    deposit_receipt_address: &Pubkey,
    owner: &Pubkey,
    vault_token_account: &Pubkey,
    destination_token_account: &Pubkey,
    fee_token_account: &Pubkey,
    deposit_stake_authority: &Pubkey,
    pool_mint: &Pubkey,
    token_program: &Pubkey,
    after_cool_down: bool,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*deposit_receipt_address, false),
        AccountMeta::new(*owner, !after_cool_down),
        AccountMeta::new(*vault_token_account, false),
        AccountMeta::new(*destination_token_account, false),
        AccountMeta::new(*fee_token_account, false),
        AccountMeta::new_readonly(*deposit_stake_authority, false),
        AccountMeta::new_readonly(*pool_mint, false),
        AccountMeta::new_readonly(*token_program, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&StakeDepositInterceptorInstruction::ClaimPoolTokens).unwrap(),
    }
}
