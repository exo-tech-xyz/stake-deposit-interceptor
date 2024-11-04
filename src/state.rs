use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;
use spl_pod::primitives::{PodU32, PodU64};

/// Variables to construct linearly decaying fees over some period of time.
#[repr(C)]
#[derive(Clone, Copy, BorshDeserialize, BorshSerialize, Debug, PartialEq, Pod, Zeroable)]
pub struct StakePoolDepositStakeAuthority {
    /// Corresponding stake pool where this PDA is the `deposit_stake_authority`
    pub stake_pool: Pubkey,
    /// Mint of the LST from the StakePool
    pub pool_mint: Pubkey,
    /// Address with control over the below parameters
    pub authority: Pubkey,
    /// TokenAccount that temporarily holds the LST minted from the StakePool
    pub vault: Pubkey,
    /// Program ID for the stake_pool
    pub stake_pool_program_id: Pubkey,
    /// The duration after a `DepositStake` in which the depositor would owe fees.
    pub cool_down_period: PodU64,
    /// The initial fee rate (in bps) proceeding a `DepositStake` (i.e. at T0).
    pub inital_fee_rate: PodU32,
    /// Owner of the fee token_account
    pub fee_wallet: Pubkey,
    /// Bump seed for derivation
    pub bump_seed: u8,
}

impl StakePoolDepositStakeAuthority {
    /// Check whether the StakePoolDepositStakeAuthority account has been initialized
    pub fn is_initialized(&self) -> bool {
        self.authority != Pubkey::default()
    }
}

/// Representation of some amount of claimable LST
#[repr(C)]
#[derive(Clone, Copy, BorshDeserialize, BorshSerialize, Debug, PartialEq, Pod, Zeroable)]
pub struct DepositReceipt {
	/// A generated seed for the PDA of this receipt
	pub base: Pubkey,
	/// Owner of the Deposit receipt who must sign to claim
	pub owner: Pubkey,
    /// StakePool the DepositReceipt originated from
	pub stake_pool: Pubkey,
	/// Timestamp of original deposit invocation
	pub deposit_time: PodU64,
	/// Total amount of claimable lst that was minted during Deposit
	pub lst_amount: PodU64,
	/// Cool down period at time of deposit.
	pub cool_down_period: PodU64,
	/// Initial fee rate at time of deposit
	pub initial_fee_rate: PodU32,
    /// Bump seed for derivation
    pub bump_seed: u8,
}