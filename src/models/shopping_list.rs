use serde_derive::{Deserialize, Serialize};
use validator::{Validate};
use mobc_postgres::tokio_postgres::Row;
use crate::models::{is_uuid};
use uuid::Uuid;

#[derive(Debug, Deserialize, Validate, Serialize, Clone)]
pub struct ShoppingList {
    id: Option<String>,
    pub title: String,
    pub description: String,
    #[validate(custom = "is_uuid")]
    pub owner: String,
}

#[derive(Debug, Deserialize, Validate, Serialize, Clone)]
pub struct PartialShoppingListDTO {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Validate, Serialize, Clone)]
pub struct PartialShoppingList {
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Query {
    pub limit: i32,
    pub page: i32,
}

impl ShoppingList {
    pub fn from_row(row: &Row) -> Self {
        let id: Uuid = row.get(0);
        let owner_id: Uuid = row.get(3);
        ShoppingList {
            id: Some(id.to_string()),
            title: row.get(1),
            description: row.get(2),
            owner: owner_id.to_string(),
        }
    }

    pub fn apply_changes(self: &mut Self, changes: &PartialShoppingList) {
        if let Some(title) = &changes.title  {
            self.title = String::from(title);
        }
        if let Some(description) = &changes.description {
            self.description = String::from(description);
        }
    }
}
