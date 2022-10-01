use warp::{Filter, Rejection, Reply};
use crate::services::items::{create_items, get_items as get_items_handler, update_item, delete_item as delete_item_handler};
use uuid::Uuid;
use crate::middlewares::{with_vec_body, with_body, with_connection, with_query};
use crate::middlewares::auth::with_auth;
use crate::models::GlobalContext;

pub fn items_router(ctx: &GlobalContext) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    patch_item(ctx)
        .or(add_items(ctx))
        .or(get_items(ctx))
        .or(delete_item(ctx))
        .or(warp::get().and(with_item_id_path()).map(|i1, i2| format!("{} {}", i1, i2)))
}

fn get_items(ctx: &GlobalContext) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone  {
    warp::get()
        .and(with_path())
        .and(with_auth(&ctx.redis_pool))
        .and(with_connection(&ctx.pg_pool))
        .and(with_query())
        .and_then(get_items_handler)
}

fn add_items(ctx: &GlobalContext) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::post()
        .and(with_path())
        .and(with_auth(&ctx.redis_pool))
        .and(with_connection(&ctx.pg_pool))
        .and(with_vec_body())
        .and_then(create_items)
}

fn patch_item(ctx: &GlobalContext) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::patch()
        .and(with_item_id_path())
        .and(with_auth(&ctx.redis_pool))
        .and(with_connection(&ctx.pg_pool))
        .and(with_body())
        .and_then(update_item)
}

fn delete_item(ctx: &GlobalContext) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::delete()
        .and(with_item_id_path())
        .and(with_auth(&ctx.redis_pool))
        .and(with_connection(&ctx.pg_pool))
        .and_then(delete_item_handler)
}

fn with_path() -> impl Filter<Extract = (Uuid,), Error = Rejection> + Copy {
    warp::path!("shopping_list" / Uuid / "item")
        .and(warp::path::end())
}

fn with_item_id_path() -> impl Filter<Extract = (Uuid,Uuid), Error = Rejection> + Copy {
    warp::path!("shopping_list" / Uuid / "item" / Uuid)
        .and(warp::path::end())
}
