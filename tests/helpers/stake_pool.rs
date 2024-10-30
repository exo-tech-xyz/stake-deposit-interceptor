use solana_program_test::ProgramTestContext;
use solana_sdk::{
    borsh1::{get_instance_packed_len, get_packed_len},
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    stake,
    system_instruction::{self},
    transaction::Transaction,
};

use super::{create_mint, create_token_account};

/// Create a stake-pool stake account
pub async fn create_stake_account(
    ctx: &mut ProgramTestContext,
    authorized: &stake::state::Authorized,
    lockup: &stake::state::Lockup,
    stake_amount: u64,
) -> Pubkey {
    let keypair = Keypair::new();
    let rent = ctx.banks_client.get_rent().await.unwrap();
    let lamports =
        rent.minimum_balance(std::mem::size_of::<stake::state::StakeStateV2>()) + stake_amount;
    let create_stake_account_ix = stake::instruction::create_account(
        &ctx.payer.pubkey(),
        &keypair.pubkey(),
        authorized,
        lockup,
        lamports,
    );
    let tx = Transaction::new_signed_with_payer(
        &create_stake_account_ix,
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &keypair],
        ctx.last_blockhash,
    );

    ctx.banks_client.process_transaction(tx).await.unwrap();

    keypair.pubkey()
}

/// Create a stake pool and all of it's dependencies including the SPL Mint.
pub async fn create_stake_pool(ctx: &mut ProgramTestContext) -> Pubkey {
    let pool_mint = create_mint(ctx).await;
    let pool_fee_account = create_token_account(ctx, &pool_mint).await;
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
    let reserve_stake_account =
        create_stake_account(ctx, &authorized, &lockup, LAMPORTS_PER_SOL).await;
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

    stake_pool_keypair.pubkey()
}
