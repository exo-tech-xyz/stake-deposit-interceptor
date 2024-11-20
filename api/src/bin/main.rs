use std::{net::SocketAddr, str::FromStr, sync::Arc};

use clap::Parser;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use tracing::info;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Bind address for the server
    #[arg(long, env, default_value_t = SocketAddr::from_str("0.0.0.0:3000").unwrap())]
    pub bind_addr: SocketAddr,

    /// RPC url
    #[arg(long, env, default_value = "https://api.mainnet-beta.solana.com")]
    pub json_rpc_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    info!("args: {:?}", args);

    info!("starting server at {}", args.bind_addr);

    let rpc_client = RpcClient::new(args.json_rpc_url.clone());
    info!("started rpc client at {}", args.json_rpc_url);

    let state = Arc::new(stake_deposit_interceptor_api::router::RouterState { rpc_client });

    let app = stake_deposit_interceptor_api::router::get_routes(state);

    let listener = tokio::net::TcpListener::bind(args.bind_addr).await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}
