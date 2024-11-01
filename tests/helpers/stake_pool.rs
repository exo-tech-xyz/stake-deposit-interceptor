use std::num::NonZeroU32;

use solana_program_test::{BanksClient, ProgramTestContext};
use solana_sdk::{
    borsh1::{get_instance_packed_len, get_packed_len, try_from_slice_unchecked},
    hash::Hash,
    msg,
    native_token::LAMPORTS_PER_SOL,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    stake,
    system_instruction::{self},
    transaction::Transaction,
    transport::TransportError,
};
use spl_stake_pool::MAX_VALIDATORS_TO_UPDATE;
use spl_token::state::Account;

use super::{create_mint, create_token_account, create_vote, get_account};

// Copied from SPL stake-pool program
pub const DEFAULT_VALIDATOR_STAKE_SEED: Option<NonZeroU32> = NonZeroU32::new(1_010);
pub const DEFAULT_TRANSIENT_STAKE_SEED: u64 = 42;

// Copied from SPL stake-pool program
pub struct ValidatorStakeAccount {
    pub stake_account: Pubkey,
    pub transient_stake_account: Pubkey,
    pub transient_stake_seed: u64,
    pub validator_stake_seed: Option<NonZeroU32>,
    pub vote: Keypair,
    pub validator: Keypair,
    pub stake_pool: Pubkey,
}

impl ValidatorStakeAccount {
    pub fn new(
        stake_pool: &Pubkey,
        validator_stake_seed: Option<NonZeroU32>,
        transient_stake_seed: u64,
    ) -> Self {
        let validator = Keypair::new();
        let vote = Keypair::new();
        let (stake_account, _) = spl_stake_pool::find_stake_program_address(
            &spl_stake_pool::id(),
            &vote.pubkey(),
            stake_pool,
            validator_stake_seed,
        );
        let (transient_stake_account, _) = spl_stake_pool::find_transient_stake_program_address(
            &spl_stake_pool::id(),
            &vote.pubkey(),
            stake_pool,
            transient_stake_seed,
        );
        ValidatorStakeAccount {
            stake_account,
            transient_stake_account,
            transient_stake_seed,
            validator_stake_seed,
            vote,
            validator,
            stake_pool: *stake_pool,
        }
    }
}

/// Get the minimum amount of lamports needed for a delegation.
pub async fn stake_get_minimum_delegation(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
) -> u64 {
    let transaction = Transaction::new_signed_with_payer(
        &[stake::instruction::get_minimum_delegation()],
        Some(&payer.pubkey()),
        &[payer],
        *recent_blockhash,
    );
    let mut data = banks_client
        .simulate_transaction(transaction)
        .await
        .unwrap()
        .simulation_details
        .unwrap()
        .return_data
        .unwrap()
        .data;
    data.resize(8, 0);
    data.try_into().map(u64::from_le_bytes).unwrap()
}

/// Create a stake-pool stake account
pub async fn create_stake_account(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    authorized: &stake::state::Authorized,
    lockup: &stake::state::Lockup,
    stake_amount: u64,
    recent_blockhash: Hash,
) -> Pubkey {
    let keypair = Keypair::new();
    let rent = banks_client.get_rent().await.unwrap();
    let lamports =
        rent.minimum_balance(std::mem::size_of::<stake::state::StakeStateV2>()) + stake_amount;
    let create_stake_account_ix = stake::instruction::create_account(
        &payer.pubkey(),
        &keypair.pubkey(),
        authorized,
        lockup,
        lamports,
    );
    let tx = Transaction::new_signed_with_payer(
        &create_stake_account_ix,
        Some(&payer.pubkey()),
        &[&payer, &keypair],
        recent_blockhash,
    );

    banks_client.process_transaction(tx).await.unwrap();

    keypair.pubkey()
}

/// Delegate stake to a specific validator.
pub async fn delegate_stake_account(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    stake: &Pubkey,
    authorized: &Keypair,
    vote: &Pubkey,
) {
    let mut transaction = Transaction::new_with_payer(
        &[stake::instruction::delegate_stake(
            stake,
            &authorized.pubkey(),
            vote,
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer, authorized], *recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
}

/// Add a Validator to a given StakePool.
pub async fn add_validator_to_pool(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    stake_pool_accounts: &StakePoolAccounts,
    staker: &Keypair,
    stake: &Pubkey,
    validator: &Pubkey,
    seed: Option<NonZeroU32>,
) {
    let instructions = vec![spl_stake_pool::instruction::add_validator_to_pool(
        &spl_stake_pool::id(),
        &stake_pool_accounts.stake_pool,
        &staker.pubkey(),
        &stake_pool_accounts.reserve_stake_account,
        &stake_pool_accounts.withdraw_authority,
        &stake_pool_accounts.validator_list,
        stake,
        validator,
        seed,
    )];
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[payer, staker],
        *recent_blockhash,
    );
    banks_client.process_transaction(transaction).await;
}

/// Holds all relevant keys for a StakePool
pub struct StakePoolAccounts {
    pub stake_pool: Pubkey,
    pub reserve_stake_account: Pubkey,
    pub pool_mint: Pubkey,
    pub withdraw_authority: Pubkey,
    pub pool_fee_account: Pubkey,
    pub validator_list: Pubkey,
}

/// Create a stake pool and all of it's dependencies including the SPL Mint.
pub async fn create_stake_pool(ctx: &mut ProgramTestContext) -> StakePoolAccounts {
    let pool_mint = create_mint(ctx).await;
    let pool_fee_account = create_token_account(ctx, &ctx.payer.pubkey(), &pool_mint).await;
    let max_validators = 5;

    let stake_pool_keypair = Keypair::new();
    let validator_list_keypair = Keypair::new();
    let rent = ctx.banks_client.get_rent().await.unwrap();
    let rent_stake_pool =
        rent.minimum_balance(get_packed_len::<spl_stake_pool::state::StakePool>());
    let validator_list_size =
        get_instance_packed_len(&spl_stake_pool::state::ValidatorList::new(max_validators))
            .unwrap();
    let rent_validator_list = rent.minimum_balance(validator_list_size);
    let zero_fee = spl_stake_pool::state::Fee {
        denominator: 100,
        numerator: 0,
    };
    let (withdraw_authority, _) = Pubkey::find_program_address(
        &[&stake_pool_keypair.pubkey().to_bytes(), b"withdraw"],
        &spl_stake_pool::id(),
    );
    // Stake account with 1 Sol from the ProgramTestContect `payer`
    let authorized = stake::state::Authorized {
        staker: withdraw_authority,
        withdrawer: withdraw_authority,
    };
    let lockup = stake::state::Lockup::default();
    let reserve_stake_account = create_stake_account(
        &mut ctx.banks_client,
        &ctx.payer,
        &authorized,
        &lockup,
        LAMPORTS_PER_SOL,
        ctx.last_blockhash,
    )
    .await;
    let create_stake_pool_account_ix = system_instruction::create_account(
        &ctx.payer.pubkey(),
        &stake_pool_keypair.pubkey(),
        rent_stake_pool,
        get_packed_len::<spl_stake_pool::state::StakePool>() as u64,
        &spl_stake_pool::id(),
    );
    let create_validator_list_account_ix = system_instruction::create_account(
        &ctx.payer.pubkey(),
        &validator_list_keypair.pubkey(),
        rent_validator_list,
        validator_list_size as u64,
        &spl_stake_pool::id(),
    );
    let update_mint_authority_ix = spl_token::instruction::set_authority(
        &spl_token::id(),
        &pool_mint,
        Some(&withdraw_authority),
        spl_token::instruction::AuthorityType::MintTokens,
        &ctx.payer.pubkey(),
        &[],
    )
    .unwrap();
    let init_stake_pool_ix = spl_stake_pool::instruction::initialize(
        &spl_stake_pool::id(),
        &stake_pool_keypair.pubkey(),
        &ctx.payer.pubkey(),
        &ctx.payer.pubkey(),
        // incorrect withdraw authority
        &withdraw_authority,
        &validator_list_keypair.pubkey(),
        &reserve_stake_account,
        &pool_mint,
        &pool_fee_account,
        &spl_token::id(),
        None,
        zero_fee,
        zero_fee,
        zero_fee,
        0,
        max_validators,
    );

    let tx = Transaction::new_signed_with_payer(
        &[
            create_stake_pool_account_ix,
            create_validator_list_account_ix,
            update_mint_authority_ix,
            init_stake_pool_ix,
        ],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &stake_pool_keypair, &validator_list_keypair],
        ctx.last_blockhash,
    );

    ctx.banks_client.process_transaction(tx).await.unwrap();

    StakePoolAccounts {
        stake_pool: stake_pool_keypair.pubkey(),
        reserve_stake_account,
        pool_mint,
        withdraw_authority,
        pool_fee_account,
        validator_list: validator_list_keypair.pubkey(),
    }
}

/// Updates the stake_deposit_authority on the given StakePool.
pub async fn update_stake_deposit_authority(
    banks_client: &mut BanksClient,
    stake_pool_accounts: &StakePoolAccounts,
    new_stake_deposit_authority: &Pubkey,
    manager: &Keypair,
    recent_blockhash: Hash,
) {
    let instruction = spl_stake_pool::instruction::set_funding_authority(
        &spl_stake_pool::id(),
        &stake_pool_accounts.stake_pool,
        &manager.pubkey(),
        Some(new_stake_deposit_authority),
        spl_stake_pool::instruction::FundingType::StakeDeposit,
    );
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&manager.pubkey()),
        &[manager],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();
}

/// Deposit Sol into the stake pool
pub async fn deposit_sol(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    stake_pool: &Pubkey,
    pool_mint: &Pubkey,
    withdraw_authority: &Pubkey,
    reserve_stake_account: &Pubkey,
    pool_fee_account: &Pubkey,
    recent_blockhash: &Hash,
    pool_mint_account: &Pubkey,
    amount: u64,
) {
    let signers = vec![payer];
    let instruction = spl_stake_pool::instruction::deposit_sol(
        &spl_stake_pool::id(),
        &stake_pool,
        withdraw_authority,
        reserve_stake_account,
        &payer.pubkey(),
        pool_mint_account,
        pool_fee_account,
        pool_fee_account,
        pool_mint,
        &spl_token::id(),
        amount,
    );
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &signers,
        *recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap()
}

// Creates a Validator and adds them to the StakePool.
pub async fn create_validator_and_add_to_pool(
    ctx: &mut ProgramTestContext,
    stake_pool_accounts: &StakePoolAccounts,
) -> ValidatorStakeAccount {
    let validator_stake = ValidatorStakeAccount::new(
        &stake_pool_accounts.stake_pool,
        DEFAULT_VALIDATOR_STAKE_SEED,
        DEFAULT_TRANSIENT_STAKE_SEED,
    );

    // Create a pool_mint account to receive the LST from DepositSol below
    let pool_token_account =
        create_token_account(ctx, &ctx.payer.pubkey(), &stake_pool_accounts.pool_mint).await;

    let rent = ctx.banks_client.get_rent().await.unwrap();
    let stake_rent = rent.minimum_balance(std::mem::size_of::<stake::state::StakeStateV2>());
    let min_delegation =
        stake_get_minimum_delegation(&mut ctx.banks_client, &ctx.payer, &ctx.last_blockhash).await;
    let current_minimum_delegation = spl_stake_pool::minimum_delegation(min_delegation);

    // Deposit sol to stake pool
    deposit_sol(
        &mut ctx.banks_client,
        &ctx.payer,
        &stake_pool_accounts.stake_pool,
        &stake_pool_accounts.pool_mint,
        &stake_pool_accounts.withdraw_authority,
        &stake_pool_accounts.reserve_stake_account,
        &stake_pool_accounts.pool_fee_account,
        &ctx.last_blockhash,
        &pool_token_account,
        current_minimum_delegation + stake_rent,
    )
    .await;

    // Create a vote
    create_vote(
        &mut ctx.banks_client,
        &ctx.payer,
        &ctx.last_blockhash,
        &validator_stake.validator,
        &validator_stake.vote,
    )
    .await;

    // Add validator to pool
    add_validator_to_pool(
        &mut ctx.banks_client,
        &ctx.payer,
        &ctx.last_blockhash,
        stake_pool_accounts,
        &ctx.payer,
        &validator_stake.stake_account,
        &validator_stake.vote.pubkey(),
        validator_stake.validator_stake_seed,
    )
    .await;

    validator_stake
}

/// Updates all validator balances and StakePool balances
pub async fn stake_pool_update_all(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    stake_pool_accounts: &StakePoolAccounts,
    recent_blockhash: &Hash,
    no_merge: bool,
) -> Option<TransportError> {
    let validator_list_account =
        get_account(banks_client, &stake_pool_accounts.validator_list).await;
    let validator_list = try_from_slice_unchecked::<spl_stake_pool::state::ValidatorList>(
        validator_list_account.data.as_slice(),
    )
    .unwrap();
    let mut instructions = vec![];
    for (i, chunk) in validator_list
        .validators
        .chunks(MAX_VALIDATORS_TO_UPDATE)
        .enumerate()
    {
        instructions.push(
            spl_stake_pool::instruction::update_validator_list_balance_chunk(
                &spl_stake_pool::id(),
                &stake_pool_accounts.stake_pool,
                &stake_pool_accounts.withdraw_authority,
                &stake_pool_accounts.validator_list,
                &stake_pool_accounts.reserve_stake_account,
                &validator_list,
                chunk.len(),
                i * MAX_VALIDATORS_TO_UPDATE,
                no_merge,
            )
            .unwrap(),
        );
    }
    instructions.extend([
        spl_stake_pool::instruction::update_stake_pool_balance(
            &spl_stake_pool::id(),
            &stake_pool_accounts.stake_pool,
            &stake_pool_accounts.withdraw_authority,
            &stake_pool_accounts.validator_list,
            &stake_pool_accounts.reserve_stake_account,
            &stake_pool_accounts.pool_fee_account,
            &stake_pool_accounts.pool_mint,
            &spl_token::id(),
        ),
        spl_stake_pool::instruction::cleanup_removed_validator_entries(
            &spl_stake_pool::id(),
            &stake_pool_accounts.stake_pool,
            &stake_pool_accounts.validator_list,
        ),
    ]);
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[payer],
        *recent_blockhash,
    );
    banks_client
        .process_transaction(transaction)
        .await
        .map_err(|e| e.into())
        .err()
}
