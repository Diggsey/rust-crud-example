// Iron web framework and middleware
extern crate iron;
extern crate mount;
extern crate logger;

// Diesel ORM with r2d2 connection pool
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
extern crate r2d2;
extern crate r2d2_diesel;

// Serde serialization framework
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;

// GraphQL
#[macro_use]
extern crate juniper;

// Misc. libraries
extern crate uuid;
extern crate dotenv;
extern crate pretty_env_logger;

// Our modules
#[macro_use]
mod macros;
mod schema;
mod routes;
mod database;

// Imports
use std::{thread, env};
use std::time::Duration;
use iron::prelude::*;
use logger::Logger;
use juniper::iron_handlers::GraphQLHandler;
use database::middleware::DatabaseWrapper;
use database::postgres::PgDatabase;
use database::interface::Database;


pub fn postgres_database() -> PgDatabase {
    // Give the SQL proxy a second to start
    thread::sleep(Duration::from_secs(1));

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    PgDatabase::connect(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

// Create database middleware
fn migrate_database() {
    postgres_database().migrate();
    println!("Up-to-date!");
}


fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init()
        .expect("Failed to initialize logger");

    for arg in env::args().skip(1) {
        match &*arg {
            "--migrate" => return migrate_database(),
            other => panic!("Unexpected argument: {}", other)
        }
    }

    let mut chain = Chain::new(routes::get());
    chain.link(Logger::new(None));
    chain.link_before(DatabaseWrapper::new(postgres_database()));

    let listener = Iron::new(chain).http("0.0.0.0:3000").unwrap();
    println!("Server started on 0.0.0.0:3000");
    drop(listener);
}
