use crate::errors::error::Error;
use serde::{ Deserialize, Serialize };
use sqlx::{ FromRow, Postgres, Transaction };
use uuid::Uuid;
use borsh::BorshDeserialize;
use solana_sdk::pubkey::Pubkey;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Badge {
    pub id: Uuid,
    pub owner: Pubkey,
    pub supply: u64,
    pub max_supply: u64,
    pub price: u64,
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub created_at: i64,
}

#[derive(Serialize, Deserialize, BorshDeserialize, Debug)]
pub struct BadgeAccount {
    pub owner: Pubkey,
    pub supply: u64,
    pub max_supply: u64,
    pub price: u64,
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub created_at: i64,
}
