use ::{
    stake_deposit_interceptor_cranker::{ InterceptorCranker, CrankerConfig },
    solana_sdk::{
        signature::{ read_keypair_file, Signer }, // Added Signer trait
        pubkey::Pubkey,
        commitment_config::CommitmentConfig,
    },
    std::{ str::FromStr, time::Duration, sync::Arc },
    dotenv::dotenv,
    tracing::{ info, Level },
};

fn load_config() -> Result<CrankerConfig, Box<dyn std::error::Error>> {
    // Load .env file
    dotenv().ok();
    // Load each environment variable with better error messages
    let rpc_url = std::env::var("RPC_URL").map_err(|_| "RPC_URL not found in environment")?;

    let ws_url = std::env::var("WS_URL").map_err(|_| "WS_URL not found in environment")?;

    let keypair_path = std::env
        ::var("KEYPAIR_PATH")
        .map_err(|_| "KEYPAIR_PATH not found in environment")?;

    let payer = Arc::new(
        read_keypair_file(&keypair_path).map_err(|_|
            format!("Failed to read keypair from {}", keypair_path)
        )?
    );

    let program_id = Pubkey::from_str(
        &std::env::var("PROGRAM_ID").map_err(|_| "PROGRAM_ID not found in environment")?
    ).map_err(|_| "Invalid PROGRAM_ID format")?;

    let interval = Duration::from_secs(
        std::env
            ::var("INTERVAL_SECONDS")
            .map_err(|_| "INTERVAL_SECONDS not found in environment")?
            .parse()
            .map_err(|_| "INTERVAL_SECONDS must be a valid number")?
    );

    Ok(CrankerConfig {
        rpc_url,
        ws_url,
        program_id,
        payer,
        interval,
        commitment: CommitmentConfig::confirmed(),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with a simpler configuration
    tracing_subscriber
        ::fmt() // Use fully qualified path
        .with_max_level(Level::INFO)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .pretty()
        .init();

    info!("Logger initialized");

    // Load .env file
    dotenv().ok();
    info!("Environment loaded");

    // Load configuration
    let config = load_config()?;

    info!("Configuration loaded successfully:");
    info!("RPC URL: {}", config.rpc_url);
    info!("WS URL: {}", config.ws_url);
    info!("Program ID: {}", config.program_id);
    info!("Payer: {}", config.payer.as_ref().pubkey()); // Signer trait now in scope
    info!("Interval: {}s", config.interval.as_secs());

    // Initialize cranker
    let cranker = InterceptorCranker::new(config);
    info!("Cranker initialized");

    // Start processing
    info!("Starting cranker service...");
    cranker.start().await;

    Ok(())
}
