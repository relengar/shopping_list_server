pub mod auth;
pub mod error;

use std::fmt::{Debug};
use validator::{Validate, ValidationErrors};
use warp::{Filter, Rejection};
use serde::de::DeserializeOwned;
use crate::middlewares::error::HttpError;
use crate::services::database::{DBPool, DBConn, RedisPool, RedisConn, get_connection};

fn validate_dto<T: Validate>(data: T) -> Result<T, HttpError> {
    match data.validate() {
        Err(e) => Err(HttpError::BadRequest(e)),
        _ => Ok(data),
    }
}

fn validate_vec_dto<T: Validate>(data: Vec<T>) -> Result<Vec<T>, HttpError> {
    let mut errors: Vec<ValidationErrors> = Vec::new();
    for item in data.iter() {
        match item.validate() {
            Err(e) => {
                errors.push(e);
            },
            _ => { },
        }
    }
    if errors.len() > 0 {
        return Err(HttpError::BadRequests(errors));
    }
    Ok(data)
}

pub fn with_body<T>() -> impl Filter<Extract = (T,), Error = Rejection> + Clone
    where
        T: DeserializeOwned + Validate + Send + Debug,
{
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
        .and_then(|value| async move { validate_dto(value).map_err(warp::reject::custom) })
}

pub fn with_query<T: 'static>() -> impl Filter<Extract = (T,), Error = Rejection> + Clone
    where
        T: DeserializeOwned + Validate + Send + Debug,
{
    warp::query()
        .and_then(|value: T| async move { validate_dto(value).map_err(warp::reject::custom) })
}

pub fn with_vec_body<T>() -> impl Filter<Extract = (Vec<T>,), Error = Rejection> + Clone
    where
        T: DeserializeOwned + Validate + Send,
{
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
        .and_then(|value| async move { validate_vec_dto(value).map_err(warp::reject::custom) })
}

pub fn with_database(db_pool: &DBPool) -> impl Filter<Extract = (DBConn,), Error = Rejection> + Clone {
    // TODO fix the double cloning
    let pool = db_pool.clone();
    warp::any().map(move || pool.clone())
        .and_then(get_connection)
}

pub fn with_redis(db_pool: &RedisPool) -> impl Filter<Extract = (RedisConn,), Error = Rejection> + Clone {
    // TODO fix the double cloning
    let pool = db_pool.clone();
    warp::any().map(move || pool.clone())
        .and_then(get_connection)
}