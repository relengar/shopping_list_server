use serde_derive::{Deserialize, Serialize};
use validator::{Validate};
use crate::models::unit::Unit;
use mobc_postgres::tokio_postgres::Row;
use std::str::FromStr;
use uuid::Uuid;
use std::convert::{TryFrom};

#[derive(Debug, Deserialize, Validate, Serialize, Clone)]
pub struct Item {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    #[serde(rename = "totalAmount")]
    #[validate(range(min = 0, max = 5))]
    pub total_amount: u32,
    #[serde(rename = "currentAmount")]
    #[validate(range(min = 0, max = 5))]
    pub current_amount: u32,
    pub unit: Unit,
    pub bought: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, Validate, Serialize, Clone)]
pub struct PartialItem {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "totalAmount")]
    #[validate(range(min = 0, max = 5))]
    pub total_amount: Option<u32>,
    #[serde(rename = "currentAmount")]
    #[validate(range(min = 0, max = 5))]
    pub current_amount: Option<u32>,
    pub unit: Option<Unit>,
    pub bought: Option<bool>,
    pub tags: Option<Vec<String>>,
}

 impl Item {
     pub fn from_row(row: &Row) -> Self {
         let id: Uuid = row.get(0);
         Item {
             id: Some(id.to_string()),
             name: row.get(1),
             description: row.get(2),
             total_amount: parse_amount(row.get(3)),
             current_amount: parse_amount(row.get(4)),
             unit: Unit::from_str(row.get(5)).unwrap(),
             bought: row.get(6),
             tags: row.get(7),
         }
     }

     pub fn apply_changes(self: &mut Self, updates: &PartialItem) {
         if let Some(name) = &updates.name  {
             self.name = String::from(name);
         }
         if let Some(description) = &updates.description  {
             self.description = String::from(description);
         }
         if let Some(total_amount) = &updates.total_amount  {
             self.total_amount = total_amount.clone();
         }
         if let Some(current_amount) = &updates.current_amount  {
             self.current_amount = current_amount.clone();
         }
         if let Some(unit) = &updates.unit  {
             self.unit = unit.clone();
         }
         if let Some(bought) = &updates.bought  {
             self.bought = bought.clone();
         }
         if let Some(tags) = &updates.tags  {
             self.tags = tags.clone();
         }
     }

     // pub fn to_query(self: &Self) -> Vec<String> {
     //     let current_amount = String::from(&self.current_amount.to_string());
     //    vec![
     //        // &self.id,
     //        // &self.name,
     //        String::from(&self.description),
     //        current_amount,
     //        // &self.total_amount.to_string(),
     //        // &self.unit.to_string(),
     //        // &self.bought.to_string(),
     //        // &format!("{{{}}}", &self.tags.join(",")),
     //    ]
     // }
 }

#[derive(Debug, Serialize)]
pub struct PartialUpdateResponse {
    pub items: Vec<Item>,
    pub errors: Vec<String>,
}

fn parse_amount(amount: i32) -> u32 {
    let attempt = u32::try_from(amount);
    match attempt {
        Ok(value) => value,
        Err(e) => {
            println!("Failed to covert value {} to u32", amount);
            panic!("{}", e)
        }
    }
}