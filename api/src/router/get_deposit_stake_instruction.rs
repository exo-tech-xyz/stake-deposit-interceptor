use axum::{response::IntoResponse, Json};

pub(crate) async fn get_deposit_stake_instruction() -> crate::Result<impl IntoResponse> {
    Ok(Json("hello Deposit Stake!"))
}
