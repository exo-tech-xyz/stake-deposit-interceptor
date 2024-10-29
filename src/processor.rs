use std::mem;

use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh1::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_pod::primitives::{PodU32, PodU64};

use crate::{
    error::StakeDepositInterceptorError,
    instruction::{
        derive_stake_pool_deposit_stake_authority, InitStakePoolDepositStakeAuthorityArgs,
        StakeDepositInterceptorInstruction, STAKE_POOL_DEPOSIT_STAKE_AUTHORITY,
    },
    state::StakePoolDepositStakeAuthority,
};

pub struct Processor;

impl Processor {
    /// Initialize the `StakePoolDepositStakeAuthority` that will be used when calculating the time
    /// decayed fees.
    pub fn process_init_stake_pool_deposit_stake_authority(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        init_deposit_stake_authority_args: InitStakePoolDepositStakeAuthorityArgs,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer_info = next_account_info(account_info_iter)?;
        let deposit_stake_authority_info: &AccountInfo<'_> = next_account_info(account_info_iter)?;
        let authority = next_account_info(account_info_iter)?;
        let stake_pool_info = next_account_info(account_info_iter)?;
        let stake_pool_manager_info = next_account_info(account_info_iter)?;
        let stake_pool_program_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;

        let rent = Rent::get()?;

        // Validate: authority and StakePool's manager signed the TX
        if !authority.is_signer || !stake_pool_manager_info.is_signer {
            return Err(StakeDepositInterceptorError::SignatureMissing.into());
        }

        // Validate: StakePool must be owned by the correct program
        if stake_pool_info.owner != stake_pool_program_info.key {
            return Err(StakeDepositInterceptorError::InvalidStakePool.into());
        }

        let stake_pool = try_from_slice_unchecked::<spl_stake_pool::state::StakePool>(
            &stake_pool_info.data.borrow(),
        )?;

        // Validate: manager is StakePool's manager
        if stake_pool.manager != *stake_pool_manager_info.key {
            return Err(StakeDepositInterceptorError::InvalidStakePoolManager.into());
        }

        let (deposit_stake_authority_pda, _bump_seed) =
            derive_stake_pool_deposit_stake_authority(program_id, stake_pool_info.key);

        if deposit_stake_authority_pda != *deposit_stake_authority_info.key {
            return Err(StakeDepositInterceptorError::InvalidSeeds.into());
        }

        // Create and initialize the StakePoolDepositStakeAuthority account
        create_pda_account(
            payer_info,
            &rent,
            mem::size_of::<StakePoolDepositStakeAuthority>(),
            program_id,
            system_program_info,
            deposit_stake_authority_info,
            &[
                STAKE_POOL_DEPOSIT_STAKE_AUTHORITY,
                &stake_pool_info.key.to_bytes(),
                &[init_deposit_stake_authority_args.bump_seed],
            ],
        )?;

        let mut deposit_stake_authority = try_from_slice_unchecked::<StakePoolDepositStakeAuthority>(
            &deposit_stake_authority_info.data.borrow(),
        )?;
        // Ensure the account has not been in use
        if deposit_stake_authority.is_initialized() {
            return Err(StakeDepositInterceptorError::AlreadyInUse.into());
        }

        // Error if StakePoolDepositStakeAuthority account is not rent exempt
        if !rent.is_exempt(
            deposit_stake_authority_info.lamports(),
            deposit_stake_authority_info.data_len(),
        ) {
            return Err(ProgramError::AccountNotRentExempt);
        }

        // Set StakePoolDepositStakeAuthority values
        deposit_stake_authority.stake_pool = *stake_pool_info.key;
        deposit_stake_authority.pool_mint = stake_pool.pool_mint;
        deposit_stake_authority.stake_pool_program_id = *stake_pool_program_info.key;
        deposit_stake_authority.authority = *authority.key;
        deposit_stake_authority.fee_wallet = init_deposit_stake_authority_args.fee_wallet;
        deposit_stake_authority.cool_down_period =
            PodU64::from_primitive(init_deposit_stake_authority_args.cool_down_period);
        deposit_stake_authority.inital_fee_rate =
            PodU32::from_primitive(init_deposit_stake_authority_args.initial_fee_rate);
        deposit_stake_authority.bump_seed = init_deposit_stake_authority_args.bump_seed;
        borsh::to_writer(
            &mut deposit_stake_authority_info.data.borrow_mut()[..],
            &deposit_stake_authority,
        )?;

        Ok(())
    }

    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = StakeDepositInterceptorInstruction::try_from_slice(input)?;
        match instruction {
            StakeDepositInterceptorInstruction::StakePoolDepositStakeAuthority(args) => {
                Self::process_init_stake_pool_deposit_stake_authority(program_id, accounts, args)?;
            }
            _ => {}
        }
        Ok(())
    }
}

/// Check account owner is the given program
fn check_account_owner(
    account_info: &AccountInfo,
    program_id: &Pubkey,
) -> Result<(), ProgramError> {
    if *program_id != *account_info.owner {
        msg!(
            "Expected account to be owned by program {}, received {}",
            program_id,
            account_info.owner
        );
        Err(ProgramError::IncorrectProgramId)
    } else {
        Ok(())
    }
}

/// Create a PDA account for the given seeds
fn create_pda_account<'a>(
    payer: &AccountInfo<'a>,
    rent: &Rent,
    space: usize,
    owner: &Pubkey,
    system_program: &AccountInfo<'a>,
    new_pda_account: &AccountInfo<'a>,
    new_pda_signer_seeds: &[&[u8]],
) -> ProgramResult {
    if new_pda_account.lamports() > 0 {
        let required_lamports = rent
            .minimum_balance(space)
            .max(1)
            .saturating_sub(new_pda_account.lamports());

        if required_lamports > 0 {
            invoke(
                &system_instruction::transfer(payer.key, new_pda_account.key, required_lamports),
                &[
                    payer.clone(),
                    new_pda_account.clone(),
                    system_program.clone(),
                ],
            )?;
        }

        invoke_signed(
            &system_instruction::allocate(new_pda_account.key, space as u64),
            &[new_pda_account.clone(), system_program.clone()],
            &[new_pda_signer_seeds],
        )?;

        invoke_signed(
            &system_instruction::assign(new_pda_account.key, owner),
            &[new_pda_account.clone(), system_program.clone()],
            &[new_pda_signer_seeds],
        )
    } else {
        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                new_pda_account.key,
                rent.minimum_balance(space).max(1),
                space as u64,
                owner,
            ),
            &[
                payer.clone(),
                new_pda_account.clone(),
                system_program.clone(),
            ],
            &[new_pda_signer_seeds],
        )
    }
}
