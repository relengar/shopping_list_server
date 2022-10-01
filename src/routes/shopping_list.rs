use warp::{Filter, Reply, Rejection};
use crate::services::shopping_list::{get_shopping_lists,create,update,delete};
use crate::middlewares::{with_body,with_connection,with_query};
use crate::middlewares::auth::{with_auth, AuthenticatedUser};
use uuid::Uuid;
use crate::models::GlobalContext;

pub fn shopping_list_router(ctx: &GlobalContext) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("shopping_list")
        .and(
            post_list(ctx)
                .or(get_my_lists(ctx))
                .or(update_list(ctx))
                .or(delete_list(ctx))
        )
    .and(warp::path::end())
}

fn get_my_lists(ctx: &GlobalContext) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::get()
        .and(with_connection(&ctx.pg_pool))
        .and(with_query())
        .and(with_auth(&ctx.redis_pool))
        .and_then(get_shopping_lists)
}

fn post_list(ctx: &GlobalContext) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::post()
        .and(with_connection(&ctx.pg_pool))
        .and(with_auth(&ctx.redis_pool))
        .and(with_body())
        .and_then(create)
}

fn update_list(ctx: &GlobalContext) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::patch()
        .and(warp::path!(Uuid))
        .and(with_auth(&ctx.redis_pool))
        .and(with_connection(&ctx.pg_pool))
        .and(with_body())
        .and_then(update)
}

fn delete_list(ctx: &GlobalContext) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::delete()
        .and(warp::path!(Uuid))
        .and(with_auth(&ctx.redis_pool).map(|user: AuthenticatedUser| user.id))
        .and(with_connection(&ctx.pg_pool))
        .and_then(delete)
}
