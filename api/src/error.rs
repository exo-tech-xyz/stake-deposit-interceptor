use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use solana_rpc_client_api::client_error::Error as RpcError;
use solana_sdk::pubkey::Pubkey;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Rpc Error")]
    RpcError(#[from] RpcError),
    #[error("Could not deserialize StakePoolDepositStakeAuthority {0}")]
    ParseStakeDepositAuthorityError(Pubkey),
    #[error("Could not deserialize StakePool {0}")]
    ParseStakePoolError(Pubkey),
    #[error("Could not deserialize Stake state {0}")]
    ParseStakeStateError(Pubkey),
    #[error("Could not deserialize Validator list {0}")]
    ParseValidatorListError(Pubkey),
    #[error("Stake voter_pubkey is invalid or missing")]
    InvalidStakeVoteAccount,
    #[error("Internal Error")]
    InternalError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    pub error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::RpcError(e) => {
                error!("Rpc error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Rpc error")
            }
            ApiError::ParseStakeDepositAuthorityError(e) => {
                error!("Parse StakePoolDepositStakeAuthority error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Pubkey StakePoolDepositStakeAuthority error")
            }
            ApiError::ParseStakePoolError(e) => {
                error!("Parse StakePool error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Pubkey StakePool error")
            }
            ApiError::ParseStakeStateError(e) => {
                error!("Parse StakeState error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Pubkey StakeState error")
            }
            ApiError::ParseValidatorListError(e) => {
                error!("Parse ValidatorList error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Pubkey ValidatorList error")
            }
            ApiError::InvalidStakeVoteAccount => (StatusCode::BAD_REQUEST, "Stake voter_pubkey is invalid or missing"),
            ApiError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"),
        };
        (
            status,
            Json(Error {
                error: error_message.to_string(),
            }),
        )
            .into_response()
    }
}
