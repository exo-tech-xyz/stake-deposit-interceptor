use std::num::NonZeroU32;

use jito_bytemuck::AccountDeserialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, stake};
use spl_stake_pool::{find_stake_program_address, find_withdraw_authority_program_address};
use stake_deposit_interceptor::{
    instruction::{
        create_deposit_stake_instruction, create_init_deposit_stake_authority_instruction,
        derive_stake_pool_deposit_stake_authority,
    },
    state::StakePoolDepositStakeAuthority,
};

use crate::{
    checked_transaction_with_signers, get_stake_pool, get_stake_state, get_validator_list,
    send_transaction, CommandResult, Config, Error,
};

macro_rules! unique_signers {
    ($vec:ident) => {
        $vec.sort_by_key(|l| l.pubkey());
        $vec.dedup();
    };
}

fn get_stake_deposit_authority(
    rpc_client: &RpcClient,
    stake_deposit_authority_address: &Pubkey,
) -> Result<StakePoolDepositStakeAuthority, Error> {
    let account_data = rpc_client.get_account_data(stake_deposit_authority_address)?;
    let stake_deposit_authority = StakePoolDepositStakeAuthority::try_from_slice_unchecked(
        account_data.as_slice(),
    )
    .map_err(|err| {
        format!(
            "Invalid stake_deposit_authority {}: {}",
            stake_deposit_authority_address, err
        )
    })?;
    Ok(*stake_deposit_authority)
}

/// Create a StakePoolStakeDepositAuthority on the
/// stake-pool-interceptor program.
pub fn command_create_stake_deposit_authority(
    config: &Config,
    stake_pool_address: &Pubkey,
    fee_wallet: &Pubkey,
    cool_down_seconds: u64,
    initial_fee_bps: u32,
    authority: Box<dyn Signer>,
) -> CommandResult {
    // Ephemeral keypair used for stake_deposit_authority PDA seed.
    let base = Keypair::new();
    let stake_pool = get_stake_pool(&config.rpc_client, stake_pool_address)?;
    let ix = create_init_deposit_stake_authority_instruction(
        &stake_deposit_interceptor::id(),
        &config.fee_payer.pubkey(),
        stake_pool_address,
        &stake_pool.pool_mint,
        &spl_stake_pool::id(),
        &spl_token::id(),
        fee_wallet,
        cool_down_seconds,
        initial_fee_bps,
        &authority.pubkey(),
        &base.pubkey(),
    );

    let (deposit_stake_authority_pubkey, _bump_seed) = derive_stake_pool_deposit_stake_authority(
        &stake_deposit_interceptor::id(),
        stake_pool_address,
        &base.pubkey(),
    );

    let base_signer: Box<dyn Signer> = Box::new(base);
    let transaction = checked_transaction_with_signers(
        config,
        &[ix],
        &[&config.fee_payer, &authority, &base_signer],
    )?;
    send_transaction(config, transaction)?;
    println!("Created stake_deposit_authority:");
    print!("{:?}", deposit_stake_authority_pubkey);
    Ok(())
}

/// Deposit a stake account through the interceptor program
pub fn command_deposit_stake(
    config: &Config,
    stake_deposit_authority_address: &Pubkey,
    stake: &Pubkey,
    withdraw_authority: Box<dyn Signer>,
    referrer_token_account: &Option<Pubkey>,
) -> CommandResult {
    let stake_deposit_authority =
        get_stake_deposit_authority(&config.rpc_client, stake_deposit_authority_address)?;

    // Most below is copy/pasta from `command_deposit_stake` with very slight modifications.
    let stake_pool = get_stake_pool(&config.rpc_client, &stake_deposit_authority.stake_pool)?;
    let stake_state = get_stake_state(&config.rpc_client, stake)?;

    let vote_account = match stake_state {
        stake::state::StakeStateV2::Stake(_, stake, _) => Ok(stake.delegation.voter_pubkey),
        _ => Err("Wrong stake account state, must be delegated to validator"),
    }?;
    // Check if this vote account has staking account in the pool
    let validator_list = get_validator_list(&config.rpc_client, &stake_pool.validator_list)?;
    let validator_stake_info = validator_list
        .find(&vote_account)
        .ok_or("Vote account not found in the stake pool")?;
    let validator_seed = NonZeroU32::new(validator_stake_info.validator_seed_suffix.into());

    // Calculate validator stake account address linked to the pool
    let (validator_stake_account, _) = find_stake_program_address(
        &spl_stake_pool::id(),
        &vote_account,
        &stake_deposit_authority.stake_pool,
        validator_seed,
    );

    println!(
        "Depositing stake {} into stake pool {} via stake_deposit_authority {}",
        stake, stake_deposit_authority.stake_pool, stake_deposit_authority_address
    );

    let mut signers = vec![config.fee_payer.as_ref(), withdraw_authority.as_ref()];

    let referrer_token_account = referrer_token_account.unwrap_or(stake_deposit_authority.vault);

    let pool_withdraw_authority = find_withdraw_authority_program_address(
        &spl_stake_pool::id(),
        &stake_deposit_authority.stake_pool,
    )
    .0;

    // Finally create interceptor instructions

    // Ephemoral keypair for PDA seed of DepositReceipt
    let deposit_receipt_base = Keypair::new();

    println!("Created DepositReceipt {}", deposit_receipt_base.pubkey());

    let ixs = create_deposit_stake_instruction(
        &stake_deposit_interceptor::id(),
        &config.fee_payer.pubkey(),
        &spl_stake_pool::id(),
        &stake_deposit_authority.stake_pool,
        &stake_pool.validator_list,
        &pool_withdraw_authority,
        stake,
        &withdraw_authority.pubkey(),
        &validator_stake_account,
        &stake_pool.reserve_stake,
        &stake_deposit_authority.vault,
        &stake_pool.manager_fee_account,
        &referrer_token_account,
        &stake_pool.pool_mint,
        &spl_token::id(),
        &deposit_receipt_base.pubkey(),
        &stake_deposit_authority.base,
    );
    signers.push(&deposit_receipt_base);

    unique_signers!(signers);

    let transaction = checked_transaction_with_signers(config, &ixs, &signers)?;
    send_transaction(config, transaction)?;
    Ok(())
}
