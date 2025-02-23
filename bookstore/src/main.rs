use controllers::{Response, SuccessResponse};
use fairings::cors::{options, Cors};
use migrator::Migrator;
use rocket::http::Status;
use sea_orm_migration::MigratorTrait;

#[macro_use] extern crate rocket;

mod migrator;
mod db;
mod entities;
mod controllers;
mod fairings;
mod auth;

pub struct AppConfig {
    db_host: String,
    db_port: String,
    db_username: String,
    db_password: String,
    db_database: String,
    jwt_secret: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            db_host: std::env::var("BOOKSTORE_DB_HOST").unwrap_or("localhost".to_string()),
            db_port: std::env::var("BOOKSTORE_DB_PORT").unwrap_or("5432".to_string()),
            db_username: std::env::var("BOOKSTORE_DB_USERNAME").unwrap_or("root".to_string()),
            db_password: std::env::var("BOOKSTORE_DB_PASSWORD").unwrap_or("0101".to_string()),
            db_database: std::env::var("BOOKSTORE_DB_DATABASE").unwrap_or("bookstore".to_string()),
            jwt_secret: std::env::var("BOOKSTORE_JWT_SECRET")
                .expect("Please set the BOOKSTORE_JWT_SECRET env variable."),
        }
    }
}
#[get("/")]
fn index() -> Response<String> {
    Ok(SuccessResponse((Status::Ok, "Hello, Rocket!".to_string())))
}

#[launch]
async fn rocket() -> _ {
    dotenvy::dotenv().ok();
    let config = AppConfig::default();
    let db = db::connect(&config).await.unwrap();
    Migrator::up(&db, None).await.unwrap();

    rocket::build()
        .attach(Cors)
        .manage(db)
        .manage(config)
        .mount("/", routes![options])
        .mount("/", routes![index])
        .mount("/auth", routes![
            controllers::auth::sign_in, controllers::auth::sign_up,controllers::auth::me
        ])
        .mount("/authors", routes![
            controllers::authors::index,
            controllers::authors::create,
            controllers::authors::show,
            controllers::authors::update,
            controllers::authors::delete
        ])
        .mount("/books", routes![
            controllers::books::index,
            controllers::books::create,
            controllers::books::show,
            controllers::books::update,
            controllers::books::delete
        ])
}