#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/38739163?s=200&v=4")]
#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
// #[macro_use]
extern crate actix_base;
extern crate failure;
#[macro_use]
extern crate maplit;

use actix_web::middleware::identity::IdentityService;
use actix_web::{http, middleware, server, App};
use diesel::PgConnection;
use std::env;

mod app_state;
mod auth;
mod auth_token;
mod consts;
mod hasher;
mod prelude;
mod time;
#[macro_use]
mod layer;
mod handlers;
mod models;
pub mod routes;
mod sanitize;
pub mod schema;
mod views;

use self::app_state::AppState;
use self::prelude::*;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
    }
}

#[derive(Fail, Debug)]
enum StartErr {
    #[fail(display = "expected DATABASE_URL env var")]
    DbExpected,

    #[fail(display = "expected .env file exists")]
    DotEnvFail,
}

fn run() -> Result<(), failure::Error> {
    dotenv::dotenv().or_err(StartErr::DotEnvFail)?;
    let db_url = env::var("DATABASE_URL").or_err(StartErr::DbExpected)?;

    create_server(db_url)?;

    Ok(())
}

fn create_server(db_url: String) -> Result<(), failure::Error> {
    env_logger::init();
    use self::app_state::DbExecutor;
    use actix_base::{SyncArbiter, System};

    let cpus = num_cpus::get();
    let system = System::new("htc-server");

    let pg = SyncArbiter::start(cpus, move || DbExecutor::new(establish_connection(&db_url)));

    let server_creator = move || {
        let state = AppState::new(pg.clone());
        App::with_state(state)
            .middleware(middleware::Logger::default())
            .middleware(
                middleware::cors::Cors::build()
                    // .allowed_origin("http://127.0.0.1:9000/")
                    // .send_wildcard()
                    .supports_credentials()
                    .allowed_methods(vec![
                        "GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS",
                    ])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .allowed_headers(vec![http::header::CONTENT_TYPE])
                    .max_age(3600)
                    .finish(),
            )
            .middleware(IdentityService::new(auth_token::TokenIdentityPolicy::new(
                "bearer".into(),
            )))
            .scope("/api", routes::scope)
    };

    let app = server::new(server_creator)
        .workers(cpus)
        .bind("0.0.0.0:9000")
        .expect("Can not bind to 127.0.0.1:9000");

    println!("Server started on http://127.0.0.1:9000");
    app.start();
    system.run();

    Ok(())
}

#[inline]
fn establish_connection(db_url: &str) -> PgConnection {
    use diesel::prelude::*;

    PgConnection::establish(&db_url).unwrap_or_else(|_| panic!("Error connecting to {}", db_url))
}
