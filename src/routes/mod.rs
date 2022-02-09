use warp::{Filter,Reply};
use crate::routes::user::user_router;
use crate::routes::items::items_router;
use crate::routes::sharing::sharing_router;
use crate::routes::shopping_list::shopping_list_router;
use crate::middlewares::error::handle_rejection;
use std::convert::Infallible;
use crate::models::GlobalContext;
use warp::cors::Builder;

pub mod shopping_list;
pub mod items;
pub mod user;
pub mod sharing;

pub fn router(ctx: &GlobalContext) -> impl Filter<Extract = impl Reply, Error = Infallible> + Clone {
    user_router(ctx)
        .or(items_router(ctx))
        .or(sharing_router(ctx))
        .or(shopping_list_router(ctx))
        .with(cors())
        .with(warp::log("debug"))
        .recover(handle_rejection)

}

fn cors() -> Builder {
    // TODO load origin from env
    // let allowed_origins = vec!["http://localhost"];
    let allowed_methods = vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"];
    let allowed_headers = vec!["0","1","2","3","4","5","6","7","8","9", "10", "Authorization", "User-Agent", "Sec-Fetch-Mode", "Content-Type", "Referer", "Origin", "Accept", "Access-Control-Request-Method", "Access-Control-Request-Headers"];

    warp::cors()
        // .allow_origins(allowed_origins)
        .allow_any_origin()
        .allow_methods(allowed_methods)
        .allow_headers(allowed_headers)
}