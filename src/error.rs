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
}
