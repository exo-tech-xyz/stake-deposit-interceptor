use spl_program_error_derive::*;

#[spl_program_error]
pub enum StakeDepositInterceptorError {
    /// 0 : A signature was missing
    #[error("Signature missing")]
    SignatureMissing,
    /// 1 : Invalid seeds for PDA
    #[error("Invalid seeds")]
    InvalidSeeds,
    /// 2 : Account already in use
    #[error("Account already in use")]
    AlreadyInUse,
    /// 3 : Invalid StakePool
    #[error("StakePool does not match other inputs")]
    InvalidStakePool,
    /// 4 : Invalid StakePool Manager
    #[error("StakePool manager is invalid")]
    InvalidStakePoolManager,
    /// 5 : Invalid Authority
    #[error("Authority is invalid")]
    InvalidAuthority,
    /// 6 : Invalid StakePoolDepositStakeAuthority
    #[error("StakePoolDepositStakeAuthority key is invalid")]
    InvalidStakePoolDepositStakeAuthority,
    /// 7 : Invalid Vault account
    #[error("Vault ATA is invalid")]
    InvalidVault,
    /// 8 : Invalid Token program account   
    #[error("Token program is invalid")]
    InvalidTokenProgram,
}
