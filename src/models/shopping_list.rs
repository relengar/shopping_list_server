use serde_derive::{Deserialize, Serialize};
use validator::{Validate};
use mobc_postgres::tokio_postgres::Row;
use crate::models::{is_uuid, Model};
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

impl Model<PartialShoppingList> for ShoppingList {
    fn apply_changes(self: &mut Self, changes: &PartialShoppingList) {
        if let Some(title) = &changes.title  {
            self.title = String::from(title);
        }
        if let Some(description) = &changes.description {
            self.description = String::from(description);
        }
    }

    fn from_row(row: &Row) -> Self {
        let id: Uuid = row.get("id");
        let owner_id: Uuid = row.get("owner_id");
        ShoppingList {
            id: Some(id.to_string()),
            title: row.get("title"),
            description: row.get("description"),
            owner: owner_id.to_string(),
        }
    }
}