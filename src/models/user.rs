use serde_derive::{Deserialize, Serialize};
use validator::{Validate};
use tokio_postgres::Row;
use uuid::Uuid;
use chrono::{Utc};
use crate::models::{Model, SqlQueryResponse};

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

struct UserUpdate {}

impl Model<UserUpdate> for User {
    fn apply_changes(self: &mut Self, _changes: &UserUpdate) {}

    fn from_row(row: &Row) -> Self {
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
}

impl SqlQueryResponse for UserResponse {
    fn from_row(row: &Row) -> Self {
        let id: Uuid = row.get("id");
        Self {
            id,
            username: row.get("username"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserResponseWithSharing {
    pub id: Uuid,
    pub username: String,
    pub sharing: bool,
}

impl SqlQueryResponse for UserResponseWithSharing {
     fn from_row(row: &Row) -> Self {
        let id: Uuid = row.get("id");
        Self {
            id,
            username: row.get("username"),
            sharing: row.get("sharing")
        }
    }
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

#[derive(Clone, Deserialize, Debug, Validate)]
pub struct SearchQuery {
    pub username: Option<String>,
    #[serde(rename = "forListId")]
    pub for_list_id: Option<Uuid>,
    #[serde(rename = "excludeShared")]
    pub exclude_shared: Option<bool>,
}