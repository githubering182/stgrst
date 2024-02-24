use actix_cors::Cors;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use apalis::redis::RedisStorage;
use env_logger::{init_from_env as init_logger_from_env, Env};
use mongodb::Client;
use std::sync::{Arc, Mutex, RwLock};
use storage::core::ArchiveJob;
use storage::routes::*;

// TODO: align bucket size and overall settings
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logger_from_env(Env::new().default_filter_or("info"));

    let client = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .unwrap();
    let database = Arc::new(RwLock::new(client.database("storage_rs")));
    let redis = RedisStorage::<ArchiveJob>::connect("redis://127.0.0.1/")
        .await
        .unwrap();
    let broker = Arc::new(Mutex::new(redis));

    let port: u16 = 8000;

    println!("Trying on port: {port}");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .supports_credentials()
            .allowed_headers(vec![
                "Content-Type",
                "Authorization",
                "Access-Control-Allow-Origin",
            ])
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"]);
        App::new()
            .app_data(Data::new(database.clone()))
            .app_data(Data::new(broker.clone()))
            .wrap(Logger::default())
            .wrap(cors)
            .service(upload)
            .service(retrieve)
            .service(produce)
    })
    .workers(4)
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
