use solana_program::pubkey::Pubkey;

/// Variables to construct linearly decaying fees over some period of time.
pub struct FeeParameters {
    /// The duration after a `DepositStake` in which the depositor would owe fees.
    pub cool_down_period: i64,
    /// The initial fee rate proceeding a `DepositStake` (i.e. at T0).
    /// Setting to u32::MAX would be a starting fee rate of 100%.
    pub inital_fee_rate: u32,
    /// Address with control over the above parameters
    pub authority: Pubkey
}
