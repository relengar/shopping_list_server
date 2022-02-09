use serde_derive::{Serialize};
use validator::ValidationError;
use uuid::Uuid;

pub mod shopping_list;
pub mod item;
pub mod unit;
pub mod user;
pub mod sharing;

use serde_derive::{Deserialize};
use crate::services::database::{DBPool, RedisPool};

pub fn is_uuid(value: &str) -> Result<(), ValidationError> {
    match Uuid::parse_str(value) {
        Err(_e) => Err(ValidationError::new("Invalid uuid")),
        _ => Ok(()),
    }
}

pub struct GlobalContext {
    pub pg_pool: DBPool,
    pub redis_pool: RedisPool,
}

#[derive(Debug, Serialize, Clone)]
pub struct QueryResponse<T> {
    items: Vec<T>,
    total: i32,
}

impl<T> QueryResponse<T> {
    pub fn new(items: Vec<T>, total: i32) -> Self {
        QueryResponse {
            items,
            total,
        }
    }
}

trait Model<Partial> {
    fn apply_changes(self: &mut Self, changes: &Partial);
}

#[derive(Copy, Clone, Deserialize)]
pub struct Pagination {
    pub limit: Option<i32>,
    pub page: Option<i32>,
}

impl Pagination {
    pub fn get_limit(self: Self, default: i32) -> i32 {
        if let Some(v) = self.limit {
            v
        } else {
            default
        }
    }

    pub fn get_page(self: Self, default: i32) -> i32 {
        if let Some(v) = self.page {
            v
        } else {
            default
        }
    }
}