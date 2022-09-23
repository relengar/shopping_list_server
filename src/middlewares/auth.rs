use warp::http::header::AUTHORIZATION;
use warp::http::HeaderMap;
use uuid::Uuid;
use warp::{Filter, Rejection};
use crate::middlewares::error::HttpError;
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use jsonwebtoken::errors::Error as TokenError;
use crate::models::user::TokenClaims;
use crate::services::database::{RedisPool, RedisConn};
use crate::middlewares::with_redis;
use mobc_redis::redis::AsyncCommands;

// const PRIVATE_KEY: &[u8] = std::env::var("JWT_KEY_PRIVATE").unwrap().as_bytes();
// const PUBLIC_KEY: String = std::env::var("JWT_KEY_PUBLIC").unwrap();
// TODO load from env v variable?
const PRIVATE_KEY: &[u8] = include_bytes!("../../jwtRS256.key");
const PUBLIC_KEY: &[u8] = include_bytes!("../../jwtRS256.key.pub");
const EXPIRATION_ENV_KEY: &str = "JWT_EXPIRE_MILLIS";

#[derive(Debug)]
pub struct AuthenticatedUser {
    pub id: Uuid
}

struct TokenData {
    pub user_id: Uuid,
    pub token_id: Uuid,
}

pub fn with_auth(redis_pool: &RedisPool) -> impl Filter<Extract = (AuthenticatedUser,), Error = Rejection> + Clone {
    warp::filters::header::headers_cloned()
        .and(with_redis(redis_pool))
        .and_then(authenticate)
}

async fn authenticate(headers: HeaderMap, mut redis: RedisConn) -> Result<AuthenticatedUser, Rejection> {
    let header = headers.get(AUTHORIZATION);
    if let Some(auth_header) = header {
        let token = auth_header.to_str()
            .map_err(|_err| HttpError::InvalidToken)?;
        let parts: Vec<&str> = token.split(" ").collect();
        let TokenData { user_id, token_id } = validate_token(parts[1])
            .map_err(|error| warp::reject::custom(error))?;
        // TODO refresh token
        let session_key = get_redis_auth_key(&user_id, Some(token_id));
        let session_user_id: Option<String> = redis.get(session_key).await.map_err(|e| HttpError::Redis(e))?;
        let session_token = session_user_id.ok_or(String::from("Session expired")).map_err(|msg| HttpError::Unauthorized(msg))?;
        if session_token != parts[1] {
            return Err(warp::reject::custom(HttpError::InvalidToken));
        }
        return Ok(AuthenticatedUser { id: user_id });
    }
    Err(warp::reject::custom(HttpError::InvalidToken))
}

pub async fn create_token(user_id: &Uuid, mut redis: RedisConn) -> Result<String, TokenError> {
    let expiration: usize = dotenv::var(EXPIRATION_ENV_KEY).unwrap().parse::<usize>().unwrap();
    let token_id = Uuid::new_v4();
    let claims = TokenClaims::new(user_id, token_id, expiration);
    let encoding_key = EncodingKey::from_rsa_pem(PRIVATE_KEY)?;
    let header = Header::new(Algorithm::RS256);
    let token = encode(&header, &claims, &encoding_key)?;
    let new_session_key = get_redis_auth_key(user_id, Some(token_id));
    
    let _: () = redis.pset_ex(new_session_key, token.as_str(), expiration).await.unwrap();
    Ok(token)
}

pub async fn delete_user_tokens(user_id: &Uuid, mut redis: RedisConn) -> Result<bool, HttpError> {
    let key_query = get_redis_auth_key(user_id, None);
    let keys: Vec<String> = redis.keys(key_query).await.map_err(|e| HttpError::Redis(e))?;
    for key in keys.iter() {
        redis.del(key).await.map_err(|e| HttpError::Redis(e))?;
    }
    Ok(true)
}

fn get_redis_auth_key(user_id: &Uuid, token_id: Option<Uuid>) -> String {
    match token_id {
        Some(kid) => format!("{}:{}", user_id, kid),
        None => format!("{}:*", user_id),
    }
}

fn validate_token(token: &str) -> Result<TokenData, HttpError> {
    let decoding_key = &DecodingKey::from_rsa_pem(PUBLIC_KEY)
        .map_err(|_err| HttpError::InternalServerError)?;
    let validation = Validation::new(Algorithm::RS256);

    let data = decode::<TokenClaims>(token, &decoding_key, &validation)
        .map_err(|_err| HttpError::Unauthorized(String::from("Invalid token")))?;
    let user_id = Uuid::parse_str(data.claims.sub.as_str())
        .map_err(|_e| HttpError::InvalidToken)?;
    let token_id = Uuid::parse_str(data.claims.kid.as_str())
        .map_err(|_e| HttpError::InvalidToken)?;
    Ok(TokenData { user_id, token_id })
}