use serde_derive::{Deserialize, Serialize};
use validator::{Validate};
use tokio_postgres::Row;
use uuid::Uuid;
use chrono::{Utc};

#[derive(Debug, Deserialize, Validate, Serialize, Clone)]
pub struct User {
    id: Option<String>,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, Serialize, Clone)]
pub struct LoginDTO {
    pub username: String,
    pub password: String,
}

impl User {
    pub fn from_row(row: &Row) -> Self {
        let uuid: Uuid = row.get(0);
        User {
            id: Some(uuid.to_string()),
            username: row.get(1),
            password: row.get(2),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct TokenClaims {
    iss: String,
    iat: usize,
    exp: usize,
    pub kid: String,
    pub sub: String,
}

impl TokenClaims {
    pub fn new(id: &Uuid, token_id: Uuid, expiration: usize) -> Self {
        Self {
            iss: String::from("shopping_list"),
            iat: Utc::now().timestamp_millis() as usize,
            exp: expiration,
            sub: id.to_string(),
            kid: token_id.to_string(),
        }
    }
}