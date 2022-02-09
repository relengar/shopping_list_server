use crate::models::shopping_list::{Query, ShoppingList, PartialShoppingList, PartialShoppingListDTO};
use crate::services::database::{DBConn};
use warp::{Reply, Rejection};
use uuid::Uuid;
use crate::models::{QueryResponse};
use warp::http::StatusCode;
use warp::reply::{Response,json};
use crate::middlewares::auth::AuthenticatedUser;
use crate::middlewares::error::HttpError;
use crate::models::sharing::ShareListBody;

pub async fn get_shopping_lists(db: DBConn, query: Query, owner: AuthenticatedUser) -> Result<impl Reply, Rejection> {
    let page = (query.page - 1) * query.limit;
    // TODO mark shared? provide owner name?
    let db_response = db.query(
        "SELECT * FROM shopping_list l
                    LEFT JOIN shopping_list_share sh ON sh.shopping_list_id=l.id
                    WHERE l.owner_id=$1 OR sh.target_user_id=$1
                    LIMIT $2::int OFFSET $3::int",
        &[&owner.id, &query.limit, &page],
    ).await.map_err(|e| HttpError::Query(e))?;
    let total_count = db.query(
        "SELECT count(l.id)::int FROM shopping_list l
                    LEFT JOIN shopping_list_share sh ON sh.shopping_list_id=l.id
                    WHERE l.owner_id=$1 OR sh.target_user_id=$1",
        &[&owner.id],
    ).await.map_err(|e| HttpError::Query(e))?;

    let total: i32 = total_count.get(0).expect("count failed").get(0);
    let shopping_lists: Vec<ShoppingList> = db_response.iter().map(|row| ShoppingList::from_row(row)).collect();
    let response: QueryResponse<ShoppingList> = QueryResponse::new(shopping_lists, total);

    Ok(json(&response))
}

pub async fn create(db: DBConn, owner: AuthenticatedUser, shopping_list: PartialShoppingListDTO) -> Result<impl Reply, Rejection> {
    let resp = db.query(
        "INSERT INTO shopping_list (id, title, description, owner_id) VALUES (uuid_generate_v4(), $1, $2, $3) RETURNING *",
        &[&shopping_list.title.as_str(), &shopping_list.description.as_str(), &owner.id]
    ).await;
    match resp {
        Ok(r) => {
            let sh_row = r.get(0).expect("insert failed");
            let created_shopping_list = ShoppingList::from_row(sh_row);

            Ok(json(&created_shopping_list))
        },
        Err(e) => Err(warp::reject::custom(HttpError::Query(e)))
    }
}

pub async fn update(id: Uuid, owner: AuthenticatedUser, db: DBConn, shopping_list: PartialShoppingList) -> Result<impl Reply, Rejection> {
    validate_shopping_list_access(&id, &owner.id, &db).await?;

    let existing = db.query(
      "SELECT * FROM shopping_list WHERE id=$1",
      &[&id]
    ).await.map_err(|e| HttpError::Query(e))?;
    let opt_row = existing.get(0);
    if let Some(row) = opt_row {
        let mut existing_shopping_list = ShoppingList::from_row(&row);
        existing_shopping_list.apply_changes(&shopping_list);

        db.query(
            "UPDATE shopping_list SET (title,description) = ($2,$3) WHERE id = $1",
            &[&id, &existing_shopping_list.title, &existing_shopping_list.description],
        ).await.map_err(|e| HttpError::Query(e))?;

        Ok(warp::reply::with_status(json(&existing_shopping_list), StatusCode::OK))
    } else {
        Err(warp::reject::not_found())
    }
}

pub async fn delete(list_id: Uuid, owner_id: Uuid, mut db: DBConn) -> Result<impl Reply, Rejection> {
    let has = has_shopping_list(&list_id, &owner_id, &db).await;
    if !has {
        let msg = String::from("Unauthorized");
        return Err(warp::reject::custom(HttpError::Unauthorized(msg)))
    }

    let transaction = db.transaction().await.map_err(|e| HttpError::Query(e))?;

    transaction.query("DELETE FROM item WHERE shopping_list_id=$1", &[&list_id]).await.map_err(|e| HttpError::Query(e))?;
    transaction.query("DELETE FROM shopping_list WHERE id=$1 AND owner_id=$2", &[&list_id, &owner_id]).await.map_err(|e| HttpError::Query(e))?;

    transaction.commit().await.map_err(|e| HttpError::Query(e))?;
    Ok(warp::reply::with_status(Response::new("".into()), StatusCode::NO_CONTENT))
}

pub async fn share_list(
    shopping_list_id: Uuid,
    share_list_body: ShareListBody,
    owner: AuthenticatedUser,
    db: DBConn,
) -> Result<impl Reply, Rejection> {
    let target_user_id = share_list_body.target_user_id;

   let has = has_shopping_list(&shopping_list_id, &owner.id, &db).await;
    if !has {
        let msg = String::from("Not allowed to share this list");
        return Err(warp::reject::custom(HttpError::Unauthorized(msg)));
    }
    let is_already_shared = is_shared(&shopping_list_id, &target_user_id, &db).await;
    if is_already_shared {
        let msg = String::from("List already shared");
        return Err(warp::reject::custom(HttpError::Conflict(msg)));
    }

    db.query(
        "INSERT INTO shopping_list_share (shopping_list_id, target_user_id) VALUES ($1, $2)",
        &[&shopping_list_id, &target_user_id],
    ).await.map_err(|e| HttpError::Query(e))?;

    Ok(warp::reply::with_status(warp::reply(),StatusCode::CREATED))
}

pub async fn stop_sharing_list(
    shopping_list_id: Uuid,
    share_list_body: ShareListBody,
    owner: AuthenticatedUser,
    db: DBConn,
) -> Result<impl Reply, Rejection> {
    let has = has_shopping_list(&shopping_list_id, &owner.id, &db).await;
    if !has {
        let msg = String::from("Not allowed to share this list");
        return Err(warp::reject::custom(HttpError::Unauthorized(msg)));
    }

    db.query(
        "DELETE FROM shopping_list_share WHERE shopping_list_id=$1 AND target_user_id=$2",
        &[&shopping_list_id, &share_list_body.target_user_id],
    ).await.map_err(|e| HttpError::Query(e))?;

    Ok(warp::reply::with_status(warp::reply(),StatusCode::NO_CONTENT))
}

pub async fn has_shopping_list(id: &Uuid, owner_id: &Uuid, db: &DBConn) -> bool {
    let response = db.query("SELECT count(id) > 0 AS has FROM shopping_list WHERE id=$1 AND owner_id=$2", &[id, owner_id]).await;
    match response {
        Ok(has_row) => {
            has_row
                .get(0)
                .map_or(false, |row| row.get("has"))
        }
        Err(_e) => {
            println!("has_shopping_list query failed {:?}", _e);
            false
        }
    }
}

async fn has_access_to_hopping_list(shopping_list_id: &Uuid, user_id: &Uuid, db: &DBConn) -> bool {
    let response = db.query(
        "SELECT count(id) > 0 AS has
                  FROM shopping_list l
                      LEFT JOIN shopping_list_share sh ON sh.shopping_list_id=l.id
                  WHERE
                      l.id=$1 AND
                      (l.owner_id=$2 OR sh.target_user_id=$2)",
        &[shopping_list_id, user_id],
    ).await;
    match response {
        Ok(has_row) => {
            has_row
                .get(0)
                .map_or(false, |row| row.get("has"))
        }
        Err(_e) => {
            println!("has_shopping_list query failed {:?}", _e);
            false
        }
    }
}

pub async fn validate_shopping_list_access(shopping_list_id: &Uuid, user_id: &Uuid, db: &DBConn) -> Result<bool, Rejection> {
    let has = has_access_to_hopping_list(shopping_list_id, user_id, db).await;
    if !has {
        let msg = String::from("Can't access this shopping list");
        return Err(warp::reject::custom(HttpError::Forbidden(msg)));
    }
    Ok(true)
}

async fn is_shared(list_id: &Uuid, user_id: &Uuid, db: &DBConn) -> bool {
    let response = db.query(
        "SELECT count(*) > 0 AS has FROM shopping_list_share WHERE shopping_list_id=$1 AND target_user_id=$2",
        &[list_id, user_id],
    ).await;
    match response {
        Ok(has_row) => {
            has_row
                .get(0)
                .map_or(false, |row| row.get("has"))
        }
        Err(_e) => {
            println!("is_shared query failed {:?}", _e);
            false
        }
    }
}

