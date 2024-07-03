use crate::errors::error::Error;
use serde::{ Deserialize, Serialize };
use sqlx::{ FromRow, Postgres, Transaction };
use crate::services::service_user::validate;
use uuid::Uuid;
use bcrypt::{ DEFAULT_COST, hash };

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserPayload {
    pub wallet_address: String,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserPayload {
    pub wallet_address: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginPayload {
    pub wallet_address: String,
}

#[derive(FromRow, Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub wallet_address: String,
    pub username: String,
}

impl User {
    pub fn new(wallet_address: String, username: String) -> Result<Self, Error> {
        validate(
            &(CreateUserPayload {
                wallet_address: wallet_address.clone(),
                username: username.clone(),
            })
        )?;

        Ok(User {
            id: Uuid::new_v4(),
            wallet_address,
            username,
        })
    }

    pub async fn save(&self, session: &mut Transaction<'_, Postgres>) -> Result<(), Error> {
        sqlx
            ::query("INSERT INTO users (id, wallet_address, username) VALUES ($1, $2, $3)")
            .bind(&self.id)
            .bind(&self.wallet_address)
            .bind(&self.username)
            .execute(session).await
            .map_err(|err| {
                let error_message = format!("Database insert failed: {}", err);
                println!("{}", error_message);
                Error::CreateUserError(error_message)
            })?;

        Ok(())
    }

    pub async fn update(&self, session: &mut Transaction<'_, Postgres>) -> Result<(), Error> {
        sqlx
            ::query("UPDATE users SET wallet_address = $1, username = $2, WHERE id = $3")
            .bind(&self.wallet_address)
            .bind(&self.username)
            .bind(&self.id)
            .execute(session).await
            .map_err(|err| {
                let error_message = format!("Database update failed: {}", err);
                println!("{}", error_message);
                Error::UpdateUserError(error_message)
            })?;

        Ok(())
    }
}
