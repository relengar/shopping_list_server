use warp::Filter;
use warp::{Reply,Rejection};
use uuid::Uuid;
use crate::services::shopping_list::{get_list_sharing, share_list, stop_sharing_list};
use crate::middlewares::auth::with_auth;
use crate::middlewares::{with_body, with_connection};
use crate::models::GlobalContext;

pub fn sharing_router(ctx: &GlobalContext) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    share_shopping_list(ctx)
        .or(remove_sharing(ctx))
        .or(get_sharing(ctx))
}

fn share_shopping_list(ctx: &GlobalContext) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::post()
        .and(with_path())
        .and(with_body())
        .and(with_auth(&ctx.redis_pool))
        .and(with_connection(&ctx.pg_pool))
        .and_then(share_list)
}

fn remove_sharing(ctx: &GlobalContext) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::delete()
        .and(with_path())
        .and(with_body())
        .and(with_auth(&ctx.redis_pool))
        .and(with_connection(&ctx.pg_pool))
        .and_then(stop_sharing_list)
}

fn get_sharing(ctx: &GlobalContext) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::get()
        .and(with_path())
        .and(with_auth(&ctx.redis_pool))
        .and(with_connection(&ctx.pg_pool))
        .and_then(get_list_sharing)
}

fn with_path() -> impl Filter<Extract = (Uuid,), Error = Rejection> + Copy {
    warp::path!("shopping_list" / Uuid / "share")
        .and(warp::path::end())
}