use crate::models::user::{User, LoginDTO, UserResponse};
use warp::{Reply,Rejection};
use crate::services::database::{DBConn, RedisConn};
use argon2::{hash_encoded, verify_encoded, Config};
use warp::hyper::StatusCode;
use crate::middlewares::error::HttpError;
use crate::middlewares::auth::{create_token, delete_user_tokens, AuthenticatedUser};
use uuid::Uuid;
use serde::Serialize;

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
    let user_response = UserResponse {
        id,
        username: sh_row.get("username"),
        token
    };
    Ok(warp::reply::with_status(warp::reply::json(&user_response), StatusCode::CREATED))
}

pub async fn login_handler(db: DBConn, redis: RedisConn, credentials: LoginDTO) -> Result<impl Reply, Rejection> {
    println!("Handling login");
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

pub async fn delete_user(db: DBConn, id: Uuid, redis: RedisConn) -> Result<impl Reply, Rejection> {
    let (redis, sql) = tokio::join!(
        delete_user_tokens(&id, redis),
        db.query_raw("DELETE FROM users WHERE id=$1", vec![id]),
    );
    redis?;
    sql.map_err(|e| HttpError::Query(e))?;

    Ok(warp::reply::with_status(warp::reply(), StatusCode::NO_CONTENT))
}

pub async fn search_user(text_input: Option<String>, db: DBConn) -> Result<impl Reply, Rejection> {
    let text = match text_input {
        Some(txt) => format!("%{}%", txt),
        None => format!("%%"),
    };
    let rows = db.query(
        "SELECT u.* FROM users u WHERE username ILIKE $1",
        &[&text]
    ).await.map_err(|e| HttpError::Query(e))?;
    let users: Vec<User> = rows.iter().map(User::from_row).collect();
    Ok(warp::reply::json(&users))
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