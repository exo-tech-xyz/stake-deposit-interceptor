use bytemuck::{Pod, Zeroable};
use jito_bytemuck::{
    AccountDeserialize,
    Discriminator,
};
use solana_program::pubkey::Pubkey;
use spl_pod::primitives::{PodU32, PodU64};

/// Discriminators for accounts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StakeDepositInterceptorDiscriminators {
    DepositStakeAuthority = 1,
    DepositReceipt = 2,
}

/// Variables to construct linearly decaying fees over some period of time.
#[repr(C)]
#[derive(Clone, Copy, AccountDeserialize, Debug, PartialEq, Pod, Zeroable)]
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

impl Discriminator for StakePoolDepositStakeAuthority {
    const DISCRIMINATOR: u8 = StakeDepositInterceptorDiscriminators::DepositStakeAuthority as u8;
}

impl StakePoolDepositStakeAuthority {
    /// Check whether the StakePoolDepositStakeAuthority account has been initialized
    pub fn is_initialized(&self) -> bool {
        self.authority != Pubkey::default()
    }
}

/// Representation of some amount of claimable LST
#[repr(C)]
#[derive(Clone, Copy, AccountDeserialize, Debug, PartialEq, Pod, Zeroable)]
pub struct DepositReceipt {
    /// A generated seed for the PDA of this receipt
    pub base: Pubkey,
    /// Owner of the Deposit receipt who must sign to claim
    pub owner: Pubkey,
    /// StakePool the DepositReceipt originated from
    pub stake_pool: Pubkey,
    /// StakePoolDepositStakeAuthority the DepositReceipt is associated with
    pub stake_pool_deposit_stake_authority: Pubkey,
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

impl Discriminator for DepositReceipt {
    const DISCRIMINATOR: u8 = StakeDepositInterceptorDiscriminators::DepositReceipt as u8;
}

impl DepositReceipt {
    /// Given a current timestamp, calculate the amount of "pool" tokens
    /// are required to be sent to the fee_wallet's token account.
    pub fn calculate_fee_amount(&self, current_timestamp: i64) -> u64 {
        let cool_down_period = u64::from(self.cool_down_period);
        let end_cool_down_time = u64::from(self.deposit_time) + cool_down_period;
        let cool_down_time_left =
            end_cool_down_time.saturating_sub(current_timestamp.unsigned_abs());
        if cool_down_time_left == 0 {
            return 0;
        }
        let fee_rate_bps =
            u64::from(u32::from(self.initial_fee_rate)) * cool_down_time_left / cool_down_period;
        let total_amount = u64::from(self.lst_amount);
        let fee_amount = total_amount
            .checked_mul(fee_rate_bps)
            .expect("overflow")
            // 10_000 is the equivalent of 100% in bps
            .checked_div(10_000)
            .expect("overflow");
        fee_amount
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_fee_amount() {
        let mut deposit_receipt = DepositReceipt {
            base: Pubkey::new_unique(),
            owner: Pubkey::new_unique(),
            stake_pool: Pubkey::new_unique(),
            stake_pool_deposit_stake_authority: Pubkey::new_unique(),
            deposit_time: PodU64::from(1_000),
            lst_amount: PodU64::from(1_000_000),
            cool_down_period: PodU64::from(1_000),
            initial_fee_rate: PodU32::from(100),
            bump_seed: 0,
        };

        // fee rate is initial rate of 100bps = 10_000
        assert_eq!(deposit_receipt.calculate_fee_amount(1_000), 10_000);
        // fee rate is half of initial rate 50bps = 5_000
        assert_eq!(deposit_receipt.calculate_fee_amount(1_500), 5_000);
        // fee rate is 25% of initial rate 25bps = 2_500
        assert_eq!(deposit_receipt.calculate_fee_amount(1_750), 2_500);
        // fee rate is 0 of initial rate 0bps = 0
        assert_eq!(deposit_receipt.calculate_fee_amount(2_000), 0);
        assert_eq!(deposit_receipt.calculate_fee_amount(2_001), 0);

        // Fee should be round down to 0
        deposit_receipt.lst_amount = PodU64::from(1);
        assert_eq!(deposit_receipt.calculate_fee_amount(1_000), 0);
    }
}
