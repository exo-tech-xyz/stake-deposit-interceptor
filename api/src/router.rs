use std::sync::Arc;

use axum::{http::StatusCode, routing::get, Router};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;

mod get_deposit_stake_instructions;

pub struct RouterState {
    pub rpc_client: RpcClient,
}

pub fn get_routes(state: Arc<RouterState>) -> Router {
    let api_routes = Router::new().route(
        "/get-deposit-stake-instructions",
        get(get_deposit_stake_instructions::get_deposit_stake_instructions),
    );

    let app = Router::new().nest("/api/v1", api_routes).fallback(fallback);

    app.with_state(state)
}

async fn fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found")
}
