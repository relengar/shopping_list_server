extern crate dotenv;
use dotenv::dotenv;
use tokio_postgres::{NoTls};
use shopping_list::services::database::{init_postgres, init_redis};
use shopping_list::routes::router;
use shopping_list::models::GlobalContext;

const PORT: u16 = 3030;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let pg_pool = init_postgres(NoTls).unwrap();
    let redis_pool = init_redis().unwrap();
    let ctx = GlobalContext {
        pg_pool,
        redis_pool,
    };

    let handlers = router(&ctx);

    warp::serve(handlers)
        .run(([0, 0, 0, 0], PORT))
        // .run(([192,168,0,104], PORT))
        .await;
}
