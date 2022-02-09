use warp::{Filter, Reply, Rejection};
use crate::services::user::{create_user, delete_user, login_handler, search_user, logout_handler};
use crate::middlewares::{with_body, with_database, with_redis};
use crate::middlewares::auth::{with_auth, AuthenticatedUser};
use crate::models::GlobalContext;

pub fn user_router(ctx: &GlobalContext) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    post_user(ctx)
        .or(search(ctx))
        .or(delete_self(ctx))
        .or(login(ctx))
        .or(logout(ctx))
}

fn post_user(ctx: &GlobalContext) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("user")
        .and(warp::path::end())
        .and(warp::post())
        .and(with_database(&ctx.pg_pool))
        .and(with_redis(&ctx.redis_pool))
        .and(with_body())
        .and_then(create_user)
}

fn login(ctx: &GlobalContext) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("user" / "login")
        .and(warp::path::end())
        .and(warp::post())
        .and(with_database(&ctx.pg_pool))
        .and(with_redis(&ctx.redis_pool))
        .and(with_body())
        .and_then(login_handler)
}

fn logout(ctx: &GlobalContext) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("user" / "logout")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_auth(&ctx.redis_pool))
        .and(with_redis(&ctx.redis_pool))
        .and_then(logout_handler)
}

fn delete_self(ctx: &GlobalContext) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("user")
        .and(warp::path::end())
        .and(warp::delete())
        .and(with_database(&ctx.pg_pool))
        .and(with_auth(&ctx.redis_pool).map(|user: AuthenticatedUser| user.id))
        .and(with_redis(&ctx.redis_pool))
        .and_then(delete_user)
}

fn search(ctx: &GlobalContext) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("user")
        .and(warp::path::end())
        .and(warp::get())
        .and(warp::query())
        .and(with_database(&ctx.pg_pool))
        .and_then(search_user)
}
