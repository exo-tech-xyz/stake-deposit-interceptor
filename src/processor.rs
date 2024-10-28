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

use crate::{
    error::StakeDepositInterceptorError,
    instruction::{
        derive_fee_parameters, InitFeeParametersArgs, StakeDepositInterceptorInstruction,
        FEE_PARAMETERS,
    },
    state::FeeParameters,
};

pub struct Processor;

impl Processor {
    /// Initialize the `FeeParameters` that will be used when calculating the time
    /// decayed fees.
    pub fn process_init_fee_params_args(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        init_fee_params_args: InitFeeParametersArgs,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer_info = next_account_info(account_info_iter)?;
        let fee_parameters_info: &AccountInfo<'_> = next_account_info(account_info_iter)?;
        let authority = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;

        let rent = Rent::get()?;

        // Validate: authority account signed the TX
        if !authority.is_signer {
            return Err(StakeDepositInterceptorError::SignatureMissing.into());
        }

        let (fee_params_pda, _bump_seed) = derive_fee_parameters(program_id);

        if fee_params_pda != *fee_parameters_info.key {
            return Err(StakeDepositInterceptorError::InvalidSeeds.into());
        }

        // Create and initialize the FeeParameters account
        create_pda_account(
            payer_info,
            &rent,
            mem::size_of::<FeeParameters>(),
            program_id,
            system_program_info,
            fee_parameters_info,
            &[FEE_PARAMETERS, &[init_fee_params_args.bump_seed]],
        )?;

        let mut fee_parameters =
            try_from_slice_unchecked::<FeeParameters>(&fee_parameters_info.data.borrow())?;
        // Ensure the account has not been in use
        if fee_parameters.is_initialized() {
            return Err(StakeDepositInterceptorError::AlreadyInUse.into());
        }

        // Error if FeeParameters account is not rent exempt
        if !rent.is_exempt(
            fee_parameters_info.lamports(),
            fee_parameters_info.data_len(),
        ) {
            return Err(ProgramError::AccountNotRentExempt);
        }

        // Set FeeParameters values
        fee_parameters.authority = *authority.key;
        fee_parameters.cool_down_period = init_fee_params_args.cool_down_period;
        fee_parameters.inital_fee_rate = init_fee_params_args.initial_fee_rate;
        fee_parameters.bump_seed = init_fee_params_args.bump_seed;
        borsh::to_writer(&mut fee_parameters_info.data.borrow_mut()[..], &fee_parameters)?;
        
        Ok(())
    }

    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = StakeDepositInterceptorInstruction::try_from_slice(input)?;
        match instruction {
            StakeDepositInterceptorInstruction::InitFeeParameters(init_fee_params_args) => {
                Self::process_init_fee_params_args(program_id, accounts, init_fee_params_args)?;
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
