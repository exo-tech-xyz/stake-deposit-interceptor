use std::{num::NonZeroU32, sync::Arc};

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use bincode::deserialize;
use futures::future;
use jito_bytemuck::AccountDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::{
    borsh1::try_from_slice_unchecked, instruction::Instruction, pubkey::Pubkey, signature::Keypair,
    signer::Signer, stake,
};
use spl_stake_pool::{
    find_stake_program_address, find_withdraw_authority_program_address,
    state::{StakePool, ValidatorList},
};
use stake_deposit_interceptor::{
    instruction::create_deposit_stake_instruction, state::StakePoolDepositStakeAuthority,
};

use crate::{error::ApiError, utils::pubkey_from_str};

use super::RouterState;

#[derive(Deserialize)]
pub(crate) struct GetDepositStakeQuery {
    #[serde(deserialize_with = "pubkey_from_str")]
    payer: Pubkey,
    #[serde(deserialize_with = "pubkey_from_str")]
    stake: Pubkey,
    #[serde(deserialize_with = "pubkey_from_str")]
    stake_deposit_authority: Pubkey,
    #[serde(deserialize_with = "pubkey_from_str")]
    withdraw_authority: Pubkey,
    referrer_token_account: Option<Pubkey>,
}

#[derive(Serialize)]
struct GetDepositStakeResponse {
    deposit_receipt_address: Pubkey,
    instructions: Vec<Instruction>,
}

// TODO could DRY up this code with cli in the future, when CLI can adopt 2.X versions of solana deps.

/// Constructs the instructions necessary to `DepositStake` via the stake-pool-interceptor program.
pub(crate) async fn get_deposit_stake_instruction(
    State(state): State<Arc<RouterState>>,
    Query(query): Query<GetDepositStakeQuery>,
) -> crate::Result<impl IntoResponse> {
    let stake_deposit_authority_account_data = state
        .rpc_client
        .get_account_data(&query.stake_deposit_authority)
        .await
        .map_err(ApiError::RpcError)?;

    let stake_deposit_authority = StakePoolDepositStakeAuthority::try_from_slice_unchecked(
        stake_deposit_authority_account_data.as_slice(),
    )
    .map_err(|_| ApiError::ParseStakeDepositAuthorityError(query.stake_deposit_authority))?;

    let stake_pool_account_data_fut = state
        .rpc_client
        .get_account_data(&stake_deposit_authority.stake_pool);
    let stake_account_data_fut = state.rpc_client.get_account_data(&query.stake);
    let (stake_pool_account_data, stake_account_data) =
        future::join(stake_pool_account_data_fut, stake_account_data_fut).await;
    let stake_pool_account_data = stake_pool_account_data.map_err(ApiError::RpcError)?;
    let stake_account_data = stake_account_data.map_err(ApiError::RpcError)?;
    let stake_pool = try_from_slice_unchecked::<StakePool>(stake_pool_account_data.as_slice())
        .map_err(|_| ApiError::ParseStakePoolError(stake_deposit_authority.stake_pool))?;
    let stake_state: stake::state::StakeStateV2 = deserialize(stake_account_data.as_slice())
        .map_err(|_| ApiError::ParseStakeStateError(query.stake))?;

    let vote_account = match stake_state {
        stake::state::StakeStateV2::Stake(_, stake, _) => Ok(stake.delegation.voter_pubkey),
        _ => Err(ApiError::InvalidStakeVoteAccount),
    }?;

    let validator_list_account_data = state
        .rpc_client
        .get_account_data(&stake_pool.validator_list)
        .await?;
    let validator_list =
        try_from_slice_unchecked::<ValidatorList>(validator_list_account_data.as_slice())
            .map_err(|_| ApiError::ParseValidatorListError(stake_pool.validator_list))?;

    let validator_stake_info = validator_list
        .find(&vote_account)
        .ok_or(ApiError::InvalidStakeVoteAccount)?;
    let validator_seed = NonZeroU32::new(validator_stake_info.validator_seed_suffix.into());
    // Calculate validator stake account address linked to the pool
    let (validator_stake_account, _) = find_stake_program_address(
        &spl_stake_pool::id(),
        &vote_account,
        &stake_deposit_authority.stake_pool,
        validator_seed,
    );

    let referrer_token_account = query
        .referrer_token_account
        .unwrap_or(stake_deposit_authority.vault);

    let pool_withdraw_authority = find_withdraw_authority_program_address(
        &spl_stake_pool::id(),
        &stake_deposit_authority.stake_pool,
    )
    .0;

    // Ephemoral keypair for PDA seed of DepositReceipt
    let deposit_receipt_base = Keypair::new();

    let ixs = create_deposit_stake_instruction(
        &stake_deposit_interceptor::id(),
        &query.payer,
        &spl_stake_pool::id(),
        &stake_deposit_authority.stake_pool,
        &stake_pool.validator_list,
        &pool_withdraw_authority,
        &query.stake,
        &query.withdraw_authority,
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

    Ok(Json(GetDepositStakeResponse {
        deposit_receipt_address: deposit_receipt_base.pubkey(),
        instructions: ixs,
    }))
}
