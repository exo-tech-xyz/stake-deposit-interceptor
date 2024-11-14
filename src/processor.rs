use std::mem;

use borsh::BorshDeserialize;
use jito_bytemuck::{AccountDeserialize, Discriminator};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh1::try_from_slice_unchecked,
    clock::Clock,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction, system_program,
    sysvar::Sysvar,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token_2022::state::{Account, Mint};

use crate::{
    deposit_receipt_signer_seeds, deposit_stake_authority_signer_seeds,
    error::StakeDepositInterceptorError,
    instruction::{
        derive_stake_deposit_receipt, derive_stake_pool_deposit_stake_authority, DepositStakeArgs,
        InitStakePoolDepositStakeAuthorityArgs, StakeDepositInterceptorInstruction,
        UpdateStakePoolDepositStakeAuthorityArgs, DEPOSIT_RECEIPT,
        STAKE_POOL_DEPOSIT_STAKE_AUTHORITY,
    },
    state::{DepositReceipt, StakePoolDepositStakeAuthority},
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
        let vault_ata_info: &AccountInfo<'_> = next_account_info(account_info_iter)?;
        let authority = next_account_info(account_info_iter)?;
        let stake_pool_info = next_account_info(account_info_iter)?;
        let stake_pool_mint_info = next_account_info(account_info_iter)?;
        let stake_pool_manager_info = next_account_info(account_info_iter)?;
        let stake_pool_program_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;
        let _associated_token_account_program_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;

        let rent = Rent::get()?;

        // Validate: System program is correct native program
        check_system_program(system_program_info.key)?;
        // Validate: StakePoolDepositStakeAuthority should be owned by system program and not initialized
        check_system_account(deposit_stake_authority_info, true)?;

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

        // Validate: stake_pool's mint is same as given account
        if stake_pool.pool_mint != *stake_pool_mint_info.key {
            return Err(StakeDepositInterceptorError::InvalidStakePool.into());
        }

        // Validate: stake_pool's mint has same token program as given program
        if stake_pool_mint_info.owner != token_program_info.key {
            return Err(StakeDepositInterceptorError::InvalidTokenProgram.into());
        }

        let (deposit_stake_authority_pda, bump_seed) =
            derive_stake_pool_deposit_stake_authority(program_id, stake_pool_info.key);

        if deposit_stake_authority_pda != *deposit_stake_authority_info.key {
            return Err(StakeDepositInterceptorError::InvalidSeeds.into());
        }

        let pda_seeds = [
            STAKE_POOL_DEPOSIT_STAKE_AUTHORITY,
            &stake_pool_info.key.to_bytes(),
            &[bump_seed],
        ];
        // Create and initialize the StakePoolDepositStakeAuthority account
        create_pda_account(
            payer_info,
            &rent,
            8 + mem::size_of::<StakePoolDepositStakeAuthority>(),
            program_id,
            system_program_info,
            deposit_stake_authority_info,
            &pda_seeds,
        )?;

        let vault_ata =
            get_associated_token_address(&deposit_stake_authority_pda, &stake_pool.pool_mint);

        // Validate: Vault must be the ATA for the StakePoolDepositStakeAuthority PDA
        if vault_ata != *vault_ata_info.key {
            return Err(StakeDepositInterceptorError::InvalidVault.into());
        }

        // Create and initialize the Vault ATA
        invoke_signed(
            &spl_associated_token_account::instruction::create_associated_token_account_idempotent(
                &payer_info.key,              // Funding account
                &deposit_stake_authority_pda, // Owner of the ATA
                &stake_pool.pool_mint,        // Mint address for the token
                token_program_info.key,
            ),
            &[
                payer_info.clone(),
                vault_ata_info.clone(),
                deposit_stake_authority_info.clone(),
                stake_pool_mint_info.clone(),
                system_program_info.clone(),
                token_program_info.clone(),
            ],
            &[&pda_seeds], // PDA's signature
        )?;

        let mut deposit_stake_authority_data =
            deposit_stake_authority_info.try_borrow_mut_data()?;
        deposit_stake_authority_data[0] = StakePoolDepositStakeAuthority::DISCRIMINATOR;
        let deposit_stake_authority = StakePoolDepositStakeAuthority::try_from_slice_unchecked_mut(
            &mut deposit_stake_authority_data,
        )
        .unwrap();

        // Set StakePoolDepositStakeAuthority values
        deposit_stake_authority.stake_pool = *stake_pool_info.key;
        deposit_stake_authority.pool_mint = stake_pool.pool_mint;
        deposit_stake_authority.vault = vault_ata;
        deposit_stake_authority.stake_pool_program_id = *stake_pool_program_info.key;
        deposit_stake_authority.authority = *authority.key;
        deposit_stake_authority.fee_wallet = init_deposit_stake_authority_args.fee_wallet;
        deposit_stake_authority.cool_down_seconds =
            init_deposit_stake_authority_args.cool_down_seconds.into();
        deposit_stake_authority.inital_fee_rate =
            init_deposit_stake_authority_args.initial_fee_rate.into();
        deposit_stake_authority.bump_seed = bump_seed;

        Ok(())
    }

    /// Update `StakePoolDepositStakeAuthority` authority, fee_wallet, cool_down_seconds, and/or initial_fee_rate.
    /// ONLY accessible by the currnet authority.
    pub fn process_update_deposit_stake_authority(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        update_deposit_stake_authority_args: UpdateStakePoolDepositStakeAuthorityArgs,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let deposit_stake_authority_info: &AccountInfo<'_> = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        let new_authority_info = next_account_info(account_info_iter).ok();

        // Validate: program owns `StakePoolDepositStakeAuthority`
        check_account_owner(deposit_stake_authority_info, program_id)?;

        // Validate: authority is signer
        if !authority_info.is_signer {
            return Err(StakeDepositInterceptorError::SignatureMissing.into());
        }

        let mut deposit_stake_authority_data =
            deposit_stake_authority_info.try_borrow_mut_data()?;
        let deposit_stake_authority = StakePoolDepositStakeAuthority::try_from_slice_unchecked_mut(
            &mut deposit_stake_authority_data,
        )
        .unwrap();

        check_deposit_stake_authority_address(
            program_id,
            deposit_stake_authority_info.key,
            &deposit_stake_authority,
        )?;

        // Validate: authority matches
        if deposit_stake_authority.authority != *authority_info.key {
            return Err(StakeDepositInterceptorError::InvalidAuthority.into());
        }

        if let Some(new_authority) = new_authority_info {
            // Validate: new_authority has also signed the transaction
            if !new_authority.is_signer {
                return Err(StakeDepositInterceptorError::SignatureMissing.into());
            }
            deposit_stake_authority.authority = *new_authority.key;
        }

        if let Some(cool_down_seconds) = update_deposit_stake_authority_args.cool_down_seconds {
            deposit_stake_authority.cool_down_seconds = cool_down_seconds.into();
        }
        if let Some(initial_fee_rate) = update_deposit_stake_authority_args.initial_fee_rate {
            deposit_stake_authority.inital_fee_rate = initial_fee_rate.into();
        }
        if let Some(fee_wallet) = update_deposit_stake_authority_args.fee_wallet {
            deposit_stake_authority.fee_wallet = fee_wallet;
        }

        Ok(())
    }

    /// Invoke the provided stake-pool program's DepositStake (or DepositStakeWithSlippage), but use
    /// the vault account from the `StakePoolDepositStakeAuthority` to custody the "pool" tokens.
    pub fn process_deposit_stake(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        deposit_stake_args: DepositStakeArgs,
        minimum_pool_tokens_out: Option<u64>,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer_info = next_account_info(account_info_iter)?;
        let stake_pool_program_info = next_account_info(account_info_iter)?;
        let deposit_receipt_info = next_account_info(account_info_iter)?;
        let stake_pool_info = next_account_info(account_info_iter)?;
        let validator_stake_list_info = next_account_info(account_info_iter)?;
        let deposit_stake_authority_info = next_account_info(account_info_iter)?;
        let withdraw_authority_info = next_account_info(account_info_iter)?;
        let stake_info = next_account_info(account_info_iter)?;
        let validator_stake_account_info = next_account_info(account_info_iter)?;
        let reserve_stake_account_info = next_account_info(account_info_iter)?;
        let pool_tokens_vault_info = next_account_info(account_info_iter)?;
        let manager_fee_info = next_account_info(account_info_iter)?;
        let referrer_fee_info = next_account_info(account_info_iter)?;
        let pool_mint_info = next_account_info(account_info_iter)?;
        let clock_info = next_account_info(account_info_iter)?;
        let stake_history_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;
        let stake_program_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;

        // Validate: System program is correct native program
        check_system_program(system_program_info.key)?;
        // Validate `StakePoolDepositStakeAuthority` is owned by current program.
        check_account_owner(deposit_stake_authority_info, program_id)?;
        // Validate: DepositReceipt should be owned by system program and not initialized
        check_system_account(deposit_receipt_info, true)?;

        // NOTE: we assume that stake-pool program makes all of the assertions that the SPL stake-pool program does.

        let deposit_stake_authority_data = deposit_stake_authority_info.try_borrow_data()?;
        let deposit_stake_authority =
            StakePoolDepositStakeAuthority::try_from_slice_unchecked(&deposit_stake_authority_data)
                .unwrap();

        // Validate StakePoolDepositStakeAuthority PDA is correct
        check_deposit_stake_authority_address(
            program_id,
            deposit_stake_authority_info.key,
            &deposit_stake_authority,
        )?;
        // Validate Vault token account to receive pool tokens is coorect.
        if pool_tokens_vault_info.key != &deposit_stake_authority.vault {
            return Err(StakeDepositInterceptorError::InvalidVault.into());
        }

        let vault_token_account_before = Account::unpack(&pool_tokens_vault_info.data.borrow())?;

        // CPI to SPL stake-pool program to invoke DepositStake with the `StakePoolDepositStakeAuthority` as the
        // `stake_deposit_authority`.
        deposit_stake_cpi(
            stake_pool_program_info,
            stake_pool_info,
            validator_stake_list_info,
            deposit_stake_authority_info,
            withdraw_authority_info,
            stake_info,
            validator_stake_account_info,
            reserve_stake_account_info,
            pool_tokens_vault_info,
            manager_fee_info,
            referrer_fee_info,
            pool_mint_info,
            token_program_info,
            clock_info,
            stake_history_info,
            stake_program_info,
            &deposit_stake_authority,
            minimum_pool_tokens_out,
        )?;

        let vault_token_account_after = Account::unpack(&pool_tokens_vault_info.data.borrow())?;
        let pool_tokens_minted =
            vault_token_account_after.amount - vault_token_account_before.amount;

        // Create the DepositReceipt

        let rent = Rent::get()?;
        let clock = Clock::get()?;

        let (deposit_receipt_pda, bump_seed) = derive_stake_deposit_receipt(
            program_id,
            &deposit_stake_args.owner,
            stake_pool_info.key,
            &deposit_stake_args.base,
        );

        // Validate: DepositReceipt should be canonical PDA
        if deposit_receipt_pda != *deposit_receipt_info.key {
            return Err(StakeDepositInterceptorError::InvalidSeeds.into());
        }

        let pda_seeds = [
            DEPOSIT_RECEIPT,
            &deposit_stake_args.owner.to_bytes(),
            &stake_pool_info.key.to_bytes(),
            &deposit_stake_args.base.to_bytes(),
            &[bump_seed],
        ];
        // Create and initialize the DepositReceipt account
        create_pda_account(
            payer_info,
            &rent,
            8 + mem::size_of::<DepositReceipt>(),
            program_id,
            system_program_info,
            deposit_receipt_info,
            &pda_seeds,
        )?;

        let mut deposit_receipt_data = deposit_receipt_info.try_borrow_mut_data()?;
        deposit_receipt_data[0] = DepositReceipt::DISCRIMINATOR;
        let deposit_receipt =
            DepositReceipt::try_from_slice_unchecked_mut(&mut deposit_receipt_data).unwrap();

        deposit_receipt.base = deposit_stake_args.base;
        deposit_receipt.owner = deposit_stake_args.owner;
        deposit_receipt.stake_pool = *stake_pool_info.key;
        deposit_receipt.stake_pool_deposit_stake_authority = *deposit_stake_authority_info.key;
        deposit_receipt.deposit_time = clock.unix_timestamp.unsigned_abs().into();
        deposit_receipt.lst_amount = pool_tokens_minted.into();
        deposit_receipt.cool_down_seconds = deposit_stake_authority.cool_down_seconds;
        deposit_receipt.initial_fee_rate = deposit_stake_authority.inital_fee_rate;
        deposit_receipt.bump_seed = bump_seed;

        Ok(())
    }

    /// Update the `owner` of the DepositReceipt, allowing a different address
    /// to receive the tokens during Claim.
    pub fn process_change_deposit_receipt_owner(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let deposit_receipt_info = next_account_info(account_info_iter)?;
        let owner_info = next_account_info(account_info_iter)?;
        let new_owner_info = next_account_info(account_info_iter)?;

        // Validate: program owns `DepositReceipt`
        check_account_owner(deposit_receipt_info, program_id)?;

        // Validate: owner must be a signer
        if !owner_info.is_signer {
            return Err(StakeDepositInterceptorError::SignatureMissing.into());
        }

        let mut deposit_receipt_data = deposit_receipt_info.try_borrow_mut_data()?;
        let deposit_receipt =
            DepositReceipt::try_from_slice_unchecked_mut(&mut deposit_receipt_data).unwrap();

        // Validate: DepositReceipt address must match expected PDA
        check_deposit_receipt_address(program_id, deposit_receipt_info.key, &deposit_receipt)?;

        // Validate: owner should match that of the DepositReceipt
        if owner_info.key != &deposit_receipt.owner {
            return Err(StakeDepositInterceptorError::InvalidDepositReceiptOwner.into());
        }

        // Update owner to new_owner
        deposit_receipt.owner = *new_owner_info.key;

        Ok(())
    }

    /// Transfers "pool" tokens to a token account owned by the DepositReceipt `owner`.
    /// If this instruction is invoked during the `cool_down_seconds`, then fees will be
    /// sent to a token account owned by the `fee_wallet`. ONLY the DepositReceipt `owner`
    /// may invoke this instruction during the `cool_down_seconds`. Once the `cool_down_seconds`
    /// has ended, the instruction is permissionless and no fees are subtracted from the
    /// depositors original amount of "pool" tokens.
    pub fn process_claim_pool_tokens(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let deposit_receipt_info = next_account_info(account_info_iter)?;
        let owner_info = next_account_info(account_info_iter)?;
        let vault_token_account_info = next_account_info(account_info_iter)?;
        let destination_token_account_info = next_account_info(account_info_iter)?;
        let fee_token_account_info = next_account_info(account_info_iter)?;
        let deposit_stake_authority_info: &AccountInfo<'_> = next_account_info(account_info_iter)?;
        let pool_mint_info: &AccountInfo<'_> = next_account_info(account_info_iter)?;
        let token_program_info: &AccountInfo<'_> = next_account_info(account_info_iter)?;
        let system_program_info: &AccountInfo<'_> = next_account_info(account_info_iter)?;

        // Validate: System program is correct native program
        check_system_program(system_program_info.key)?;

        // Validate: program owns `StakePoolDepositStakeAuthority`
        check_account_owner(deposit_stake_authority_info, program_id)?;

        // Validate: program owns `DepositReceipt`
        check_account_owner(deposit_receipt_info, program_id)?;

        {
            let clock = Clock::get()?;

            let deposit_receipt_data = deposit_receipt_info.try_borrow_data()?;
            let deposit_receipt =
                DepositReceipt::try_from_slice_unchecked(&deposit_receipt_data).unwrap();

            let cool_down_end_time = u64::from(deposit_receipt.deposit_time)
                .checked_add(deposit_receipt.cool_down_seconds.into())
                .expect("overflow") as i64;

            // Validate: Owner must be signer during cool down to prevent unintended fee payment
            if cool_down_end_time > clock.unix_timestamp && !owner_info.is_signer {
                return Err(StakeDepositInterceptorError::ActiveCooldown.into());
            }

            // Validate: Owner must match that of DepositReceipt
            if &deposit_receipt.owner != owner_info.key {
                return Err(StakeDepositInterceptorError::InvalidDepositReceiptOwner.into());
            }

            let deposit_stake_authority_data = deposit_stake_authority_info.try_borrow_data()?;
            let deposit_stake_authority = StakePoolDepositStakeAuthority::try_from_slice_unchecked(
                &deposit_stake_authority_data,
            )
            .unwrap();

            // Validate: StakePoolDepositStakeAuthority PDA is correct
            check_deposit_stake_authority_address(
                program_id,
                deposit_stake_authority_info.key,
                &deposit_stake_authority,
            )?;

            // Validate: DepositReceipt address must match expected PDA
            check_deposit_receipt_address(program_id, deposit_receipt_info.key, &deposit_receipt)?;

            // Validate: StakePoolDepositStakeAuthority must match the same during creation of DepositReceipt
            if deposit_stake_authority_info.key
                != &deposit_receipt.stake_pool_deposit_stake_authority
            {
                return Err(
                    StakeDepositInterceptorError::InvalidStakePoolDepositStakeAuthority.into(),
                );
            }

            // Validate: Vault token account must match that of the `StakePoolDepositStakeAuthority`
            if &deposit_stake_authority.vault != vault_token_account_info.key {
                return Err(StakeDepositInterceptorError::InvalidVault.into());
            }

            let fee_token_account = Account::unpack(&fee_token_account_info.data.borrow())?;

            // Validate: Fee token account must be owned by `fee_wallet`
            if fee_token_account.owner != deposit_stake_authority.fee_wallet {
                return Err(StakeDepositInterceptorError::InvalidFeeTokenAccount.into());
            }

            let destination_token_account =
                Account::unpack(&destination_token_account_info.data.borrow())?;

            // Validate: Destination token account must be owned by DepositRecipt `owner`
            if destination_token_account.owner != deposit_receipt.owner {
                return Err(StakeDepositInterceptorError::InvalidDestinationTokenAccount.into());
            }

            let pool_mint = Mint::unpack(&pool_mint_info.data.borrow())?;

            let fee_amount = deposit_receipt.calculate_fee_amount(clock.unix_timestamp);

            // Transfer fee tokens to fee token account
            transfer_tokens_cpi(
                token_program_info.clone(),
                vault_token_account_info.clone(),
                pool_mint_info.clone(),
                fee_token_account_info.clone(),
                deposit_stake_authority_info.clone(),
                fee_amount,
                pool_mint.decimals,
                &deposit_stake_authority,
            )?;

            let amount = u64::from(deposit_receipt.lst_amount).saturating_sub(fee_amount);
            // Transfer the rest of the tokens to the destination token account
            transfer_tokens_cpi(
                token_program_info.clone(),
                vault_token_account_info.clone(),
                pool_mint_info.clone(),
                destination_token_account_info.clone(),
                deposit_stake_authority_info.clone(),
                amount,
                pool_mint.decimals,
                &deposit_stake_authority,
            )?;
        }

        // Close the DepositReceipt account
        close_account(deposit_receipt_info, owner_info)?;

        Ok(())
    }

    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = StakeDepositInterceptorInstruction::try_from_slice(input)?;
        match instruction {
            StakeDepositInterceptorInstruction::InitStakePoolDepositStakeAuthority(args) => {
                Self::process_init_stake_pool_deposit_stake_authority(program_id, accounts, args)?;
            }
            StakeDepositInterceptorInstruction::UpdateStakePoolDepositStakeAuthority(args) => {
                Self::process_update_deposit_stake_authority(program_id, accounts, args)?;
            }
            StakeDepositInterceptorInstruction::DepositStake(args) => {
                Self::process_deposit_stake(program_id, accounts, args, None)?;
            }
            StakeDepositInterceptorInstruction::DepositStakeWithSlippage(args) => {
                let deposit_stake_args = DepositStakeArgs {
                    base: args.base,
                    owner: args.owner,
                };
                Self::process_deposit_stake(
                    program_id,
                    accounts,
                    deposit_stake_args,
                    Some(args.minimum_pool_tokens_out),
                )?;
            }
            StakeDepositInterceptorInstruction::ChangeDepositReceiptOwner => {
                Self::process_change_deposit_receipt_owner(program_id, accounts)?;
            }
            StakeDepositInterceptorInstruction::ClaimPoolTokens => {
                Self::process_claim_pool_tokens(program_id, accounts)?;
            }
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

/// Check system program address
fn check_system_program(program_id: &Pubkey) -> Result<(), ProgramError> {
    if *program_id != system_program::id() {
        msg!(
            "Expected system program {}, received {}",
            system_program::id(),
            program_id
        );
        Err(ProgramError::IncorrectProgramId)
    } else {
        Ok(())
    }
}

/// Checks the account is owned by the System program and does not have any existing data.
fn check_system_account(account_info: &AccountInfo, is_writable: bool) -> Result<(), ProgramError> {
    if account_info.owner.ne(&system_program::id()) {
        msg!("Account is not owned by the system program");
        return Err(ProgramError::InvalidAccountOwner);
    }

    if !account_info.data_is_empty() {
        msg!("Account data is not empty");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if is_writable && !account_info.is_writable {
        msg!("Account is not writable");
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
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
        // someone can transfer lamports to accounts before they're initialized
        // in that case, creating the account won't work.
        // in order to get around it, you need to find the account with enough lamports to be rent exempt,
        // then allocate the required space and set the owner to the current program
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

/// Invokes the `DepositStake` instruction for the given stake-pool program.
fn deposit_stake_cpi<'a>(
    program_info: &AccountInfo<'a>,
    stake_pool_info: &AccountInfo<'a>,
    validator_list_storage_info: &AccountInfo<'a>,
    stake_pool_deposit_authority_info: &AccountInfo<'a>,
    stake_pool_withdraw_authority_info: &AccountInfo<'a>,
    deposit_stake_address_info: &AccountInfo<'a>,
    validator_stake_account_info: &AccountInfo<'a>,
    reserve_stake_account_info: &AccountInfo<'a>,
    pool_tokens_to_info: &AccountInfo<'a>,
    manager_fee_account_info: &AccountInfo<'a>,
    referrer_pool_tokens_account_info: &AccountInfo<'a>,
    pool_mint_info: &AccountInfo<'a>,
    token_program_id_info: &AccountInfo<'a>,
    sysvar_clock_info: &AccountInfo<'a>,
    sysvar_stake_history: &AccountInfo<'a>,
    stake_program_info: &AccountInfo<'a>,
    deposit_stake_authority: &StakePoolDepositStakeAuthority,
    minimum_pool_tokens_out: Option<u64>,
) -> Result<(), ProgramError> {
    let account_infos = vec![
        stake_pool_info.clone(),
        validator_list_storage_info.clone(),
        stake_pool_deposit_authority_info.clone(),
        stake_pool_withdraw_authority_info.clone(),
        deposit_stake_address_info.clone(),
        validator_stake_account_info.clone(),
        reserve_stake_account_info.clone(),
        pool_tokens_to_info.clone(),
        manager_fee_account_info.clone(),
        referrer_pool_tokens_account_info.clone(),
        pool_mint_info.clone(),
        sysvar_clock_info.clone(),
        sysvar_stake_history.clone(),
        token_program_id_info.clone(),
        stake_program_info.clone(),
    ];
    let accounts = vec![
        AccountMeta::new(*stake_pool_info.key, false),
        AccountMeta::new(*validator_list_storage_info.key, false),
        AccountMeta::new_readonly(*stake_pool_deposit_authority_info.key, true),
        AccountMeta::new_readonly(*stake_pool_withdraw_authority_info.key, false),
        AccountMeta::new(*deposit_stake_address_info.key, false),
        AccountMeta::new(*validator_stake_account_info.key, false),
        AccountMeta::new(*reserve_stake_account_info.key, false),
        AccountMeta::new(*pool_tokens_to_info.key, false),
        AccountMeta::new(*manager_fee_account_info.key, false),
        AccountMeta::new(*referrer_pool_tokens_account_info.key, false),
        AccountMeta::new(*pool_mint_info.key, false),
        AccountMeta::new_readonly(*sysvar_clock_info.key, false),
        AccountMeta::new_readonly(*sysvar_stake_history.key, false),
        AccountMeta::new_readonly(*token_program_id_info.key, false),
        AccountMeta::new_readonly(*stake_program_info.key, false),
    ];

    let data;
    if let Some(minimum_pool_tokens_out) = minimum_pool_tokens_out {
        data = borsh::to_vec(
            &spl_stake_pool::instruction::StakePoolInstruction::DepositStakeWithSlippage {
                minimum_pool_tokens_out,
            },
        )
        .unwrap()
    } else {
        data =
            borsh::to_vec(&spl_stake_pool::instruction::StakePoolInstruction::DepositStake).unwrap()
    }
    let ix = Instruction {
        program_id: *program_info.key,
        accounts,
        data,
    };
    let ret = invoke_signed(
        &ix,
        &account_infos,
        &[deposit_stake_authority_signer_seeds!(
            deposit_stake_authority
        )],
    );
    ret
}

/// Check the validity of the supplied deposit_stake_authority given the relevant seeds.
pub fn check_deposit_stake_authority_address(
    program_id: &Pubkey,
    deposit_stake_authority_address: &Pubkey,
    deposit_stake_authority: &StakePoolDepositStakeAuthority,
) -> Result<(), ProgramError> {
    let address = Pubkey::create_program_address(
        deposit_stake_authority_signer_seeds!(deposit_stake_authority),
        program_id,
    )?;
    if address != *deposit_stake_authority_address {
        return Err(StakeDepositInterceptorError::InvalidStakePoolDepositStakeAuthority.into());
    }
    Ok(())
}

/// Check the validity of the supplied DepositReceipt given the relevant seeds.
pub fn check_deposit_receipt_address(
    program_id: &Pubkey,
    deposit_receipt_address: &Pubkey,
    deposit_receipt: &DepositReceipt,
) -> Result<(), ProgramError> {
    let address =
        Pubkey::create_program_address(deposit_receipt_signer_seeds!(deposit_receipt), program_id)?;
    if address != *deposit_receipt_address {
        return Err(StakeDepositInterceptorError::InvalidDepositReceipt.into());
    }
    Ok(())
}

/// Transfer tokens using SPL Token or Token2022 based on the given token program.
pub fn transfer_tokens_cpi<'a>(
    token_program: AccountInfo<'a>,
    source: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    amount: u64,
    decimals: u8,
    deposit_stake_authority: &StakePoolDepositStakeAuthority,
) -> Result<(), ProgramError> {
    let ix = spl_token_2022::instruction::transfer_checked(
        token_program.key,
        source.key,
        mint.key,
        destination.key,
        authority.key,
        &[],
        amount,
        decimals,
    )?;
    invoke_signed(
        &ix,
        &[source, mint, destination, authority],
        &[deposit_stake_authority_signer_seeds!(
            deposit_stake_authority
        )],
    )
}

/// Close an account and send any leftover lamports to the destination account.
pub fn close_account<'a>(
    source: &AccountInfo<'a>,
    destination: &AccountInfo<'a>,
) -> Result<(), ProgramError> {
    let dest_starting_lamports = destination.lamports();
    **destination.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(source.lamports())
        .unwrap();
    **source.lamports.borrow_mut() = 0;

    source.assign(&system_program::ID);
    source.realloc(0, false).map_err(Into::into)
}
