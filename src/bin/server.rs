#[macro_use]
extern crate rocket;
use apalis::redis::{connect, RedisStorage};
use mongodb::Client;
use rocket::{http::Method, Config, Error};
use rocket_cors::{AllowedOrigins, CorsOptions};
use storage::{core::ArchiveJob, routes::*};

#[rocket::main]
async fn main() -> Result<(), Error> {
    let mongo = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .unwrap();

    let redis_conn = connect("redis://127.0.0.1/").await.unwrap();
    let redis = RedisStorage::<ArchiveJob>::new(redis_conn);

    let mut app_config = Config::default();
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true);

    app_config.port = 8000;
    app_config.workers = 1;

    rocket::custom(app_config)
        .manage(mongo)
        .manage(redis)
        .attach(cors.to_cors().unwrap())
        .mount("/storage/", routes![upload, retrieve])
        .mount("/worker/", routes![produce])
        .launch()
        .await?;

    Ok(())
}
