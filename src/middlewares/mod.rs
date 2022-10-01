pub mod auth;
pub mod error;

use std::fmt::{Debug};
use validator::{Validate, ValidationErrors};
use warp::{Filter, Rejection};
use serde::de::DeserializeOwned;
use crate::middlewares::error::HttpError;
use crate::services::database::{get_connection};
use mobc::{Pool, Manager, Connection};

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

pub fn with_connection<M:Manager>(db_pool: &Pool<M>) -> impl Filter<Extract = (Connection<M>,), Error = Rejection> + Clone
where <M as Manager>::Error: std::fmt::Debug
{
    let pool = db_pool.to_owned();
    warp::any()
        .map(move || pool.clone())
        .and_then(|cloned_pool| get_connection(cloned_pool, "redis"))
}