use crate::models::user::{User, LoginDTO, UserResponse, SearchQuery, UserResponseWithSharing};
use warp::{Reply,Rejection};
use crate::services::database::{DBConn, RedisConn};
use argon2::{hash_encoded, verify_encoded, Config};
use warp::hyper::StatusCode;
use crate::middlewares::error::HttpError;
use crate::middlewares::auth::{create_token, delete_user_tokens, AuthenticatedUser};
use uuid::Uuid;
use serde::Serialize;
use tokio_postgres::types::ToSql;
use crate::models::{Pagination, QueryResponse, SqlQueryResponse};
use crate::services::shopping_list::has_shopping_list;

pub async fn create_user(db: DBConn, redis: RedisConn, user: User) -> Result<impl Reply, Rejection>  {
    let config = Config::default();
    let salt = dotenv::var("SALT").unwrap();
    let hash = hash_encoded(&user.password.as_bytes(), salt.as_bytes(), &config).unwrap();

    let has_user_response = db.query("SELECT count(u.username) > 0 as has_user FROM users u WHERE u.username=$1", &[&user.username])
        .await.map_err(|e| HttpError::Query(e))?;
    let has_user: bool = has_user_response.get(0).expect("User count by name failed").get("has_user");
    if has_user {
        let msg = String::from("User already exists");
        return Err(warp::reject::custom(HttpError::Conflict(msg)));
    }

    let resp = db.query(
        "INSERT INTO users (id, username, password) VALUES (uuid_generate_v4(), $1, $2) RETURNING *",
        &[&user.username.as_str(), &hash.as_str()]
    ).await.map_err(|e| HttpError::Query(e))?;

    let sh_row = resp.get(0).expect("insert failed");
    let id = sh_row.get("id");
    let token = get_token(id, redis).await?;
    let user_response = TokenResponse {
        id,
        token,
    };
    Ok(warp::reply::with_status(warp::reply::json(&user_response), StatusCode::CREATED))
}

pub async fn login_handler(db: DBConn, redis: RedisConn, credentials: LoginDTO) -> Result<impl Reply, Rejection> {
    let resp = db.query(
        "SELECT id, password FROM users WHERE username=$1",
        &[&credentials.username.as_str()]
    ).await.map_err(|e| warp::reject::custom(HttpError::Query(e)))?;
    let row = resp.get(0)
        .ok_or(false)
        .map_err(|_f| warp::reject::custom(HttpError::Unauthorized(String::from("Invalid username"))))?;

    let hash: String = row.get(1);

    let is_password_valid = verify_encoded(&hash, credentials.password.as_bytes())
        .map_err(reject_password)?;

    match is_password_valid {
        true => {
            let id: Uuid = row.get(0);
            let token_response = get_token(id, redis).await.map(|token| TokenResponse{ token, id })?;
            Ok(warp::reply::with_status(
                warp::reply::json(&token_response),
                StatusCode::OK,
            ))
        },
        false => Err(reject_password("")),
    }
}

pub async fn logout_handler(user: AuthenticatedUser, redis: RedisConn) -> Result<impl Reply, Rejection> {
    delete_user_tokens(&user.id, redis).await?;
    Ok(warp::reply())
}

pub async fn get_by_id(db: DBConn, user: AuthenticatedUser) -> Result<impl Reply, Rejection> {
    let resp = db.query(
        "SELECT * FROM users WHERE id=$1",
        &[&user.id],
    ).await.map_err(|e| HttpError::Query(e))?;

    let option = resp.get(0);
    if let Some(row) = option {
        let user_response = UserResponse::from_row(row);
        Ok(warp::reply::with_status(
            warp::reply::json(&user_response),
            StatusCode::OK,
        ))
    } else {
        let error_message = String::from("User not found");
        let not_found_error = HttpError::NotFound(error_message);
        Err(warp::reject::custom(not_found_error))
    }
}

pub async fn delete_user(db: DBConn, id: Uuid, redis: RedisConn) -> Result<impl Reply, Rejection> {
    let (redis, sql) = tokio::join!(
        delete_user_tokens(&id, redis),
        db.query_raw("DELETE FROM users WHERE id=$1", vec![id]),
    );
    redis?;
    sql.map_err(|e| HttpError::Query(e))?;

    Ok(warp::reply::with_status(warp::reply(), StatusCode::NO_CONTENT))
}

pub async fn search_user(
    current_user: AuthenticatedUser,
    filters: SearchQuery,
    pagination: Pagination,
    db: DBConn,
) -> Result<impl Reply, Rejection> {
    let limit = pagination.get_limit(10);
    let offset = pagination.get_offset();
    let text = match filters.username {
        Some(txt) => format!("%{}%", txt),
        None => format!("%%"),
    };


    if let Some(id) = filters.for_list_id {
        let has = has_shopping_list(&id, &current_user.id, &db).await;
        if !has {
            let msg = String::from("Not allowed to view this list data");
            return Err(warp::reject::custom(HttpError::Unauthorized(msg)));
        }
    }

    let mut shared_condition = "";
    if let Some(exclude_shared) = filters.exclude_shared {
        if exclude_shared {
            shared_condition = "AND NOT (sls.target_user_id notnull AND sls.target_user_id = u.id)"
        }
    }

    let query = format!("
        SELECT
            u.id,
            u.username,
            (sls.target_user_id notnull AND sls.target_user_id = u.id) as sharing
        FROM users u
        LEFT JOIN shopping_list_share sls ON u.id = sls.target_user_id AND sls.shopping_list_id = $3
            WHERE
                u.username ILIKE $1
                AND NOT u.id = $2
                {}
        LIMIT $4::int
        OFFSET $5::int
    ", shared_condition);
    let params: &[&(dyn ToSql + Sync)] = &[
        &text,
        &current_user.id,
        &filters.for_list_id,
        &limit,
        &offset,
    ];

    let count_query = format!("
        SELECT count(u.id)::int as total 
        FROM users u
        LEFT JOIN shopping_list_share sls ON u.id = sls.target_user_id AND sls.shopping_list_id = $3
        WHERE
            u.username ILIKE $1
            AND NOT u.id = $2
            {}
    ", shared_condition);
    let count_params: &[&(dyn ToSql + Sync)] = &[
        &text,
        &current_user.id,
        &filters.for_list_id,
    ];

    let (rows, total_count) = tokio::join!(
        db.query(query.as_str(), params),
        db.query(count_query.as_str(), count_params),
    );

    let parsed_rows = rows.map_err(|e| HttpError::Query(e))?;
    let parsed_count = total_count.map_err(|e| HttpError::Query(e))?;

    let users: Vec<UserResponseWithSharing> = parsed_rows.iter().map(UserResponseWithSharing::from_row).collect();
    let total: i32 = parsed_count.get(0).expect("count failed").get("total");

    let response = QueryResponse::new(users, total);
    Ok(warp::reply::json(&response))
}

async fn get_token(id: Uuid, redis: RedisConn) -> Result<String, Rejection> {
    let token = create_token(&id, redis).await.map_err(|_e| {
        println!("Token creation failed {:?}", _e);
        warp::reject::custom(HttpError::InternalServerError)
    })?;

    Ok(token)
}

fn reject_password<T>(_e: T) -> Rejection {
    let password_error_message = String::from("invalid password");
    let password_error = HttpError::Unauthorized(password_error_message);
    warp::reject::custom(password_error)
}

#[derive(Serialize)]
struct TokenResponse {
    pub token: String,
    pub id: Uuid,
}