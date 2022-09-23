pub mod routes;
pub mod services;
pub mod models;
pub mod middlewares;

pub fn register_cancel_handler() {
    let resp = ctrlc::set_handler(move || {
        println!("Shutting down");
        std::process::exit(0);
    });

    match resp {
        Ok(_) => {},
        Err(e) => {
            println!("Could not register cancel handler {:?}", e);
            std::process::exit(1);
        }
    }
}