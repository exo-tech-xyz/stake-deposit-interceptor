#[macro_export]
macro_rules! deposit_stake_authority_signer_seeds {
    ($deposit_stake_authority:expr) => {
        &[
            STAKE_POOL_DEPOSIT_STAKE_AUTHORITY,
            $deposit_stake_authority.stake_pool.as_ref(),
            &[$deposit_stake_authority.bump_seed],
        ]
    };
}

#[macro_export]
macro_rules! deposit_receipt_signer_seeds {
    ($deposit_receipt:expr) => {
        &[
            DEPOSIT_RECEIPT,
            $deposit_receipt.owner.as_ref(),
            $deposit_receipt.stake_pool.as_ref(),
            $deposit_receipt.base.as_ref(),
            &[$deposit_receipt.bump_seed],
        ]
    };
}