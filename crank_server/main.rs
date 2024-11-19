// use solana_client::rpc_client::RpcClient;
// use solana_sdk::pubkey::Pubkey;
// use borsh::{BorshDeserialize, BorshSerialize};
// use std::str::FromStr;
// use std::{thread, time::Duration};

// // Define your DepositReceipt structure
// #[derive(BorshDeserialize, BorshSerialize, Debug)]
// pub struct DepositReceipt {
//     pub base: Pubkey,
//     pub owner: Pubkey,
//     pub stake_pool: Pubkey,
//     pub stake_pool_deposit_stake_authority: Pubkey,
//     pub deposit_time: u64,
//     pub lst_amount: u64,
//     pub cool_down_seconds: u64,
//     pub initial_fee_bps: u32,
//     pub bump_seed: u8,
// }

fn main() {
}
//     // Initialize the RPC client
//     let rpc_url = "https://api.mainnet-beta.solana.com"; // Use the appropriate network
//     let client = RpcClient::new(rpc_url.to_string());

//     // Replace with your program's public key
//     let program_id = Pubkey::from_str("YourProgramPublicKeyHere").unwrap();

//     // Fetch all accounts owned by the program
//     let accounts = client.get_program_accounts(&program_id).unwrap();

//     for (pubkey, account) in accounts {
//         // Attempt to deserialize the account data into a DepositReceipt
//         if let Ok(deposit_receipt) = DepositReceipt::try_from_slice(&account.data) {
//             println!("Found DepositReceipt at {}: {:?}", pubkey, deposit_receipt);
//         }
//     }

//     thread::sleep(Duration::from_secs(1 * 60));
// }