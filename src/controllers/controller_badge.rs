use crate::errors::error::Error;
use crate::database::db::AppState;
use crate::models::model_badge::{ Badge, BadgeAccount };
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;
use axum::{ extract::{ Path, State }, http::StatusCode, Json };
use borsh::{ BorshSerialize, BorshDeserialize };
use solana_sdk::signature::{ Keypair, Signer };
use solana_sdk::transaction::Transaction;
use solana_sdk::pubkey::Pubkey;
use solana_client::rpc_client::RpcClient;

// @route GET /api/badges
// @desc Get all badges
// @access Public
pub async fn get_all_badges(State(app_state): State<Arc<AppState>>) -> Result<
    (StatusCode, Json<Vec<BadgeAccount>>),
    Error
> {
    let rpc_url = std::env::var("RPC_URL").expect("RPC_URL must be set");
    let program_id = Pubkey::from_str(
        std::env::var("PROGRAM_ID").expect("PROGRAM_ID must be set").as_str()
    ).unwrap();
    let rpc_client = RpcClient::new(rpc_url.to_string());
    let accounts = rpc_client.get_program_accounts(&program_id).unwrap();

    let mut badges = vec![];

    for (pubkey, account) in accounts {
        // Skip the 8-byte discriminator
        let badge_data = &account.data[8..];

        // Try to deserialize and print how many bytes were read
        let (badge, _) = match BadgeAccount::deserialize(&mut &badge_data[..]) {
            Ok(badge) => (Some(badge), badge_data.len()),
            Err(_) => (None, 0),
        };

        if let Some(badge) = badge {
            println!("Account Pubkey: {:?}", pubkey);
            println!("Account Data Length: {:?}", account.data.len());
            println!("Badge Data: {:?}", badge);
            badges.push(badge);
        }
    }

    Ok((StatusCode::OK, Json(badges)))
}

// // Replace with the public key of the wallet you want to check
// let wallet_pubkey = "2g9K42Pt5y58cejTHFLhqoQWKDUcB3s3AnGESmV9ySBW".to_string();
// let pubkey = Pubkey::from_str(&wallet_pubkey).unwrap();

// match client.get_balance(&pubkey) {
//     Ok(balance) => {
//         println!("Wallet balance: {} lamports", balance);
//         println!("Wallet balance: {} SOL", (balance as f64) / 1_000_000_000.0);
//     }
//     Err(err) => eprintln!("Error fetching balance: {:?}", err),
// }
