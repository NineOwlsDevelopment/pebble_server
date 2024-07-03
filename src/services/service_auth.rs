use crate::database::db::AppState;
use crate::errors::error::Error;
use crate::services::service_user::fetch_user_by_id;
use serde::{ Deserialize, Serialize };
use chrono::{ Utc, Duration };
use std::sync::Arc;
use uuid::Uuid;
use jsonwebtoken::{ encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey };
use dotenv::dotenv;

use axum::{
    middleware::Next,
    extract::State,
    http::{ HeaderMap, header, Request },
    response::Response,
    body::Body,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: Uuid,
    exp: usize,
    iat: usize,
}

pub struct JwtTokens {
    pub access_token: String,
    pub refresh_token: String,
}
// Create short lived access token
pub fn create_access_token(user_id: &Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    dotenv().ok();

    let now = Utc::now();
    let iat = now.timestamp();
    let expires_at = now + Duration::hours(1);

    let my_claims = Claims {
        sub: user_id.to_owned(),
        exp: expires_at.timestamp() as usize,
        iat: iat as usize,
    };

    let token = encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_ref())
    )?;

    Ok(token)
}

// Create long lived refresh token
pub async fn create_refresh_token(user_id: &Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    dotenv().ok();

    let now = Utc::now();
    let iat = now.timestamp();
    let expires_at = now + Duration::days(30);

    let my_claims = Claims {
        sub: user_id.to_owned(),
        exp: expires_at.timestamp() as usize,
        iat: iat as usize,
    };

    let token = encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_ref())
    )?;

    let _ = save_token_to_db(&token, user_id).await;

    Ok(token)
}

// Save refresh token to db
pub async fn save_token_to_db(token: &str, user_id: &Uuid) -> Result<(), Error> {
    dotenv().ok();

    let db = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap()).await.unwrap();

    let query = sqlx
        ::query("INSERT INTO refresh_tokens (user_id, token) VALUES ($1, $2)")
        .bind(user_id)
        .bind(token)
        .execute(&db).await;

    match query {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::InternalServerError),
    }
}

// Verify access token
pub fn verify_access_token(token: &str) -> Result<Claims, Error> {
    dotenv().ok();

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_ref()),
        &Validation::new(Algorithm::HS256)
    );

    match token_data {
        Ok(token_data) => Ok(token_data.claims),
        Err(_) => Err(Error::InvalidToken),
    }
}

// Verify refresh token
pub async fn verify_refresh_token(token: &str) -> Result<Claims, Error> {
    dotenv().ok();

    // check if refresh token exist in db
    let db = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap()).await.unwrap();

    let is_valid_token = sqlx
        ::query_as::<_, (Uuid, String)>(
            "SELECT user_id, token FROM refresh_tokens WHERE token = $1"
        )
        .bind(token)
        .fetch_one(&db).await;

    match is_valid_token {
        Ok((_user_id, _)) => {
            let token_data = decode::<Claims>(
                token,
                &DecodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_ref()),
                &Validation::new(Algorithm::HS256)
            );

            match token_data {
                Ok(token_data) => Ok(token_data.claims),
                Err(_) => Err(Error::InvalidToken),
            }
        }
        Err(_) => Err(Error::InvalidToken),
    }
}

// Parse the cookies from the request and return the access and refresh tokens
pub fn parse_cookies_from_request(req: &Request<Body>) -> Result<JwtTokens, Error> {
    let cookies = req
        .headers()
        .get(header::COOKIE)
        .ok_or_else(|| Error::Unauthorized("No cookies provided".to_string()))?
        .to_str()
        .unwrap();

    let cookie_map: std::collections::HashMap<&str, &str> = cookies
        .split("; ")
        .map(|c| {
            let mut parts = c.splitn(2, "=");
            (parts.next().unwrap(), parts.next().unwrap())
        })
        .collect();

    let access_token = cookie_map
        .get("access_token")
        .ok_or_else(|| Error::Unauthorized("No access token provided".to_string()))?;

    let refresh_token = cookie_map
        .get("refresh_token")
        .ok_or_else(|| Error::Unauthorized("No refresh token provided".to_string()))?;

    Ok(JwtTokens {
        access_token: access_token.to_string(),
        refresh_token: refresh_token.to_string(),
    })
}

// Middleware to authenticate user on each protected route call
pub async fn auth(
    State(app_state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next
) -> Result<Response<Body>, Error> {
    let tokens = parse_cookies_from_request(&req)?;

    // Check if access token is valid
    // then check the expiration date
    // if expired, check if refresh token is valid
    // then check the expiration date
    // if expired, return unauthorized
    let token_data = verify_access_token(&tokens.access_token)?;
    if token_data.exp > (Utc::now().timestamp() as usize) {
        let user = fetch_user_by_id(token_data.sub.clone(), &app_state.db).await?;
        let new_access_token = create_access_token(&user.id).map_err(|e|
            Error::Unauthorized(e.to_string())
        )?;

        let mut headers = HeaderMap::new();
        headers.append(
            header::SET_COOKIE,
            format!("access_token={}; HttpOnly; Path=/; SameSite=Strict", new_access_token)
                .parse()
                .unwrap()
        );

        req.extensions_mut().insert(user);
        return Ok(next.run(req).await);
    }

    let token_data = verify_refresh_token(&tokens.refresh_token).await?;
    if token_data.exp > (Utc::now().timestamp() as usize) {
        let user = fetch_user_by_id(token_data.sub.clone(), &app_state.db).await?;
        let new_access_token = create_access_token(&user.id).map_err(|e|
            Error::Unauthorized(e.to_string())
        )?;

        let mut headers = HeaderMap::new();
        headers.append(
            header::SET_COOKIE,
            format!("access_token={}; HttpOnly; Path=/; SameSite=Strict", new_access_token)
                .parse()
                .unwrap()
        );

        req.extensions_mut().insert(user);
        return Ok(next.run(req).await);
    }

    Err(Error::Unauthorized("Token expired".to_string()))
}
