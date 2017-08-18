// Iron web framework and middleware
extern crate iron;
extern crate router;
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
use std::thread;
use std::time::Duration;
use iron::prelude::*;
use logger::Logger;
use database::middleware::DatabaseMiddleware;

// Create database middleware
pub fn postgres_middleware() -> DatabaseMiddleware {
    use std::env;
    use database::postgres::PgDatabase;

    // Give the SQL proxy a second to start
    thread::sleep(Duration::from_secs(1));

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let database = PgDatabase::connect(&database_url)
        .expect(&format!("Error connecting to {}", database_url));
    
    DatabaseMiddleware::new(database)
}


fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init()
        .expect("Failed to initialize logger");

    let mut chain = Chain::new(routes::get());
    chain.link(Logger::new(None));
    chain.link_before(postgres_middleware());

    let listener = Iron::new(chain).http("0.0.0.0:3000").unwrap();
    println!("Server started on 0.0.0.0:3000");
    drop(listener);
}
