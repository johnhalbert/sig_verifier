extern crate serde_derive;
extern crate redis;

use actix_web::{App, HttpServer, middleware::Logger, web::Data};
use std::sync::{Arc, Mutex};

pub mod account;
pub mod i_redis;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::var("RUST_LOG").expect("RUST_LOG");
    env_logger::init();

    let port = std::env::var("ACTIX_PORT").unwrap_or("8080".to_string());
    let port = port.parse::<u16>().unwrap_or(8080);
    let redis = Arc::new(Mutex::new(crate::i_redis::Redis::new().unwrap()));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(Data::new(redis.clone()))
            .service(account::register)
            .service(account::verify_signature)
            .service(account::verification_status)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
