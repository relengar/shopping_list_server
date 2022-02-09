use crate::models::item::{Item, PartialUpdateResponse, PartialItem};
use crate::services::database::{DBConn};
use warp::{reply, reject, Rejection};
use crate::models::{Pagination};
use uuid::Uuid;
use crate::services::shopping_list::{validate_shopping_list_access};
use warp::http::StatusCode;
use std::convert::TryFrom;
use crate::middlewares::error::HttpError;
use crate::middlewares::auth::AuthenticatedUser;

pub async fn get_items(shopping_list_id: Uuid, owner: AuthenticatedUser, db: DBConn, pagination: Pagination) -> Result<impl warp::Reply, Rejection> {
    validate_shopping_list_access(&shopping_list_id, &owner.id, &db).await?;

    let mut data = Vec::new();
    let limit = pagination.get_limit(10);
    let offset = pagination.get_page(0) * limit;
    let rows = db.query(
        "SELECT * FROM item WHERE shopping_list_id=$1 LIMIT $2::int OFFSET $3::int",
        &[&shopping_list_id, &limit, &offset],
    ).await.map_err(|error| reject::custom(HttpError::Query(error) ))?;

    for row in rows.iter() {
        let one_item = Item::from_row(row);
        data.push(one_item);
    }

    Ok(reply::json(&data))
}

pub async fn create_items(id: Uuid, owner: AuthenticatedUser, db: DBConn, items: Vec<Item>) -> Result<impl warp::Reply, Rejection> {
    validate_shopping_list_access(&id, &owner.id, &db).await?;

    let mut rows: Vec<Item> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    for item in items.iter() {
        let current_amount = i32::try_from(item.current_amount).expect("Failed to cast current_amount");
        let total_amount = i32::try_from(item.total_amount).expect("Failed to cast total_amount");
        let item_rows = db.query(
            "
                INSERT INTO item (id, name, description, current_amount, total_amount, bought, unit, tags, shopping_list_id)
                VALUES (uuid_generate_v4(), $1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING *
            ",
            &[
                        &item.name,
                        &item.description,
                        &current_amount,
                        &total_amount,
                        &item.bought,
                        &item.unit.to_string().as_str(),
                        &item.tags,
                        &id,
                    ]
        ).await;

        match item_rows {
            Ok(item) => {
                for item_row in item {
                    let item = Item::from_row(&item_row);
                    rows.push(item);
                }
            }
            Err(e) => {
                println!("Failed to insert item {} because of Error: {:?}", item.name, e);
                errors.push(format!("Insert failed for item {}", item.name))
            }
        }
    }

    let response = PartialUpdateResponse {
        items: rows,
        errors
    };
    Ok(warp::reply::with_status( reply::json(&response), StatusCode::OK))
}

pub async fn update_item(shopping_list_id: Uuid, item_id: Uuid, owner: AuthenticatedUser, db: DBConn, item: PartialItem) -> Result<impl warp::Reply, Rejection> {
    validate_shopping_list_access(&shopping_list_id, &owner.id, &db).await?;

    let db_resp = db.query("SELECT * FROM item WHERE id=$1", &[&item_id]).await.expect("Item get query failed");
    let existing_row = db_resp.get(0);
    if let Some(row) = existing_row {
        let mut existing = Item::from_row(row);
        existing.apply_changes(&item);
        let current_amount = i32::try_from(existing.current_amount).expect("Failed to cast current_amount");
        let total_amount = i32::try_from(existing.total_amount).expect("Failed to cast total_amount");

        let update_request = db.query(
            "
            UPDATE item SET (name, description, current_amount, total_amount, bought, unit, tags)
                =($1, $2, $3, $4, $5, $6, $7) WHERE id=$8
            ",
            &[
                &existing.name,
                &existing.description,
                &current_amount,
                &total_amount,
                &existing.bought,
                &existing.unit.to_string().as_str(),
                &existing.tags,
                &item_id,
            ]
        ).await;
        match update_request {
            Ok(_r) => Ok(reply::with_status(reply::json(&existing), StatusCode::OK)),
            Err(e) => Err(warp::reject::custom(HttpError::Query(e))),
        }
    } else {
        Err(warp::reject::not_found())
    }
}

pub async fn delete_item(shopping_list_id: Uuid, item_id: Uuid, owner: AuthenticatedUser, db: DBConn) -> Result<impl warp::Reply, Rejection> {
    validate_shopping_list_access(&shopping_list_id, &owner.id, &db).await?;

    let query_result = db.query("DELETE FROM item WHERE id=$1", &[&item_id]).await;
    match query_result {
        Ok(_row) => {
            Ok(reply::with_status(reply::json(&()), StatusCode::NO_CONTENT))
        },
        Err(e) => Err(warp::reject::custom(HttpError::Query(e)))
    }
}
