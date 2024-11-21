use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use solana_rpc_client_api::client_error::Error as RpcError;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Rpc Error")]
    RpcError(#[from] RpcError),
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
