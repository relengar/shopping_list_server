use tokio_postgres::{NoTls, Config, Error};
use mobc_postgres::PgConnectionManager;
use mobc_redis::RedisConnectionManager;
use mobc_redis::{redis};
use mobc::{Pool, Connection, Manager};
use std::time::Duration;
use warp::{Rejection};
use crate::middlewares::error::HttpError;

pub type DBPool = Pool<PgConnectionManager<NoTls>>;
pub type DBConn = Connection<PgConnectionManager<NoTls>>;
pub type RedisPool = Pool<RedisConnectionManager>;
pub type RedisConn = Connection<RedisConnectionManager>;

const DB_POOL_MAX_OPEN: u64 = 32;
const DB_POOL_MAX_IDLE: u64 = 8;
const DB_POOL_TIMEOUT_SECONDS: u64 = 15;

pub fn init_postgres(tls: NoTls) -> std::result::Result<DBPool, mobc::Error<Error>> {
    // let config = Config::from_str("postgres://user:passwd@localhost:5432").unwrap();
    let mut config = Config::new();
    config.password(std::env::var("DB_PASSWORD").unwrap().as_str());
    config.user(std::env::var("DB_USER").unwrap().as_str());
    config.host(std::env::var("DB_HOSTNAME").unwrap().as_str());
    config.dbname(std::env::var("DB_NAME").unwrap().as_str());

    let manager = PgConnectionManager::new(config, tls);

    Ok(Pool::builder()
        .max_open(DB_POOL_MAX_OPEN)
        .max_idle(DB_POOL_MAX_IDLE)
        .get_timeout(Some(Duration::from_secs(DB_POOL_TIMEOUT_SECONDS)))
        .build(manager)
    )
}

pub fn init_redis() -> Result<RedisPool, mobc::Error<Error>> {
    let url = create_redis_url();
    let client = redis::Client::open(url).unwrap();
    let manager = RedisConnectionManager::new(client);

    Ok(Pool::builder().build(manager))
}

pub async fn get_connection<M: Manager>(pool: Pool<M>, label: &str) -> Result<Connection<M>, Rejection>
    where <M as Manager>::Error: std::fmt::Debug
 {
    match pool.get().await {
        Ok(conn) => Ok(conn),
        Err(e) => {
            println!("Db connection error on {}: {:?}", label, e);
            Err(warp::reject::custom(HttpError::InternalServerError))
        }
    }
}

fn create_redis_url() -> String {
    let host = std::env::var("REDIS_HOST").unwrap();
    let port = std::env::var("REDIS_PORT").unwrap();
    let username = std::env::var("REDIS_USER").unwrap_or_default();
    let password = std::env::var("REDIS_PASSWORD").unwrap_or_default();

    let has_creds = username.len() > 0 && password.len() > 0;

    if has_creds {
        format!("redis://{}:{}@{}:{}", username, password, host, port)
    } else {
        format!("redis://@{}:{}", host, port)
    }
}