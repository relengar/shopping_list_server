use serde_derive::{Serialize};
use validator::ValidationError;
use validator::{Validate};
use uuid::Uuid;
use mobc_postgres::tokio_postgres::Row;

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

pub trait SqlQueryResponse {
    fn from_row(row: &Row) -> Self;
}

pub trait Model<Partial> {
    fn apply_changes(self: &mut Self, changes: &Partial);
    fn from_row(row: &Row) -> Self;
}

#[derive(Copy, Clone, Deserialize, Debug, Validate)]
pub struct Pagination {
    #[validate(range(min = 1, max = 2000))]
    pub limit: Option<i32>,
    #[validate(range(min = 1))]
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

    pub fn get_offset(self: Self) -> i32 {
        let limit = self.get_limit(1);
        if let Some(v) = self.page {
            (v - 1) * limit
        } else {
            0
        }
    }
}