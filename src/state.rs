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
        if self.authority != Pubkey::default() {
            return true;
        }
        return false;
    }
}
