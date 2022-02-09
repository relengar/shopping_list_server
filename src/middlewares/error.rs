use validator::{ValidationErrors};
use serde_derive::Serialize;
use warp::{Rejection, Reply};
use warp::reject::{Reject, MethodNotAllowed};
use std::convert::Infallible;
use warp::http::StatusCode;
use tokio_postgres::Error as TokioError;
use mobc_redis::redis::RedisError;

#[derive(Debug)]
pub enum HttpError {
    BadRequest(ValidationErrors),
    BadRequests(Vec<ValidationErrors>),
    Query(TokioError),
    Redis(RedisError),
    InvalidToken,
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    InternalServerError,
}

impl Reject for HttpError {}

#[derive(Serialize)]
struct HttpErrorBody<'s, T> {
    pub message: &'s str,
    pub code: &'s str,
    pub data: Option<Vec<T>>,
}

pub async fn handle_rejection(rejection: Rejection) -> Result<impl Reply, Infallible> {
    println!("rejection is {:?}", rejection);
    let status;
    let message;
    let mut validation_errors : Option<Vec<ValidationErrors>> = None;
    if rejection.is_not_found() {
        status = StatusCode::NOT_FOUND;
        message = "Not found";
    }
    else if let Some(e) = rejection.find::<HttpError>() {
        let (new_status, new_message, errors) = error_match(e);
        println!("Error: {:?}", e);
        validation_errors = errors;
        status = new_status;
        message = new_message;
    }
    else if let Some(_) = rejection.find::<MethodNotAllowed>() {
        status = StatusCode::NOT_FOUND;
        message = "Not found";
    }
    else {
        println!("Error {:?}", rejection);
        status = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal server error";
    }

    let error = HttpErrorBody {
        message,
        code: status.as_str(),
        data: validation_errors,
    };
    let reply = warp::reply::json(&error);
    Ok(warp::reply::with_status(reply, status))
}

fn error_match(error: &HttpError) -> (StatusCode, &str, Option<Vec<ValidationErrors>>) {
    match error {
        HttpError::BadRequest(e) => {
            let mut errors = Vec::new();
            errors.push(e.to_owned());
            (StatusCode::BAD_REQUEST, "Invalid input", Some(errors))
        }
        HttpError::BadRequests(e) => {
            let errors = Some(e.to_owned());
            (StatusCode::BAD_REQUEST, "Invalid input", errors)
        }
        HttpError::Query(e) => {
            println!("SQL Error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error", None)
        }
        HttpError::Redis(e) => {
            println!("Redis Error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error", None)
        }
        HttpError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token", None),
        HttpError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message.as_str(), None),
        HttpError::Forbidden(message) => (StatusCode::FORBIDDEN, message.as_str(), None),
        HttpError::NotFound(message) => (StatusCode::NOT_FOUND, message.as_str(), None),
        HttpError::Conflict(message) => (StatusCode::CONFLICT, message.as_str(), None),
        HttpError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error", None),
    }
}
