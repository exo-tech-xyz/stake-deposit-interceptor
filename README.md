# Stake Deposit Interceptor

A Solana program intended to become the `stake_deposit_authority` of a StakePool deployed via the SPL stake-pool program. This program allows an admin to set a fee (in basis points) on the LST received from DepositStake that linearly decays to zero over a set number of seconds. This mechanism disincentivizes the immediate transfer of the LST after deposit.

More information in the Jito governance forum [here](https://forum.jito.network/t/jip-9-adopt-interceptor-liquidity-defense/444).

## State

```rust
// PDA derived from stake_pool pubkey
pub struct StakePoolDepositStakeAuthority {
    /// A generated seed for the PDA of this receipt
    pub base: Pubkey,
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
    pub cool_down_seconds: PodU64,
    /// The initial fee rate (in bps) proceeding a `DepositStake` (i.e. at T0).
    pub inital_fee_bps: PodU32,
    /// Owner of the fee token_account
    pub fee_wallet: Pubkey,
    /// Bump seed for derivation
    pub bump_seed: u8,
}
```

```rust
// PDA derived from owner, stake_pool, and base (a randomly generated pubkey)
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
		pub cool_down_seconds: PodU64,
		/// Initial fee rate at time of deposit
		pub initial_fee_bps: PodU32,
		/// Bump seed for derivation
		pub bump_seed: u8,
}
```

## Instructions

### InitStakePoolDepositStakeAuthority

*Must be signed by the StakePool’s manager as that is the key that has control over the `stake_deposit_authority`.*

*Sets the initial authority of* StakePoolDepositStakeAuthority *along with the time decay parameters of the fees.*

### UpdateStakePoolDepositStakeAuthority

*Allows the current authority to change the authority, fee_wallet, cool_down_period, and/or initial_fee_rate.*

### DepositStake

*Invokes the DepositStake instruction of the provided StakePool program. Instead of immediately minting the jitoSol to the depositor, it is held by the interceptor program until the ClaimDeposit Instruction is called. Creates a DepositReceipt.*

### DepositStakeWithSlippage

*Same logic as `DepositStake` with an added check for slippage based on an instruction argument.*

### ClaimDeposit

*Validates DepositReceipt owner. Transfers the calculated fees to the fee_wallet and then transfers the remaining amount to the owner’s supplied token account.*

### UpdateOwner

*Let the owner of the DepositReceipt update who can claim the tokens.*
