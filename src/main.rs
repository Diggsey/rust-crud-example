// Iron web framework and middleware
extern crate iron;

// Misc. libraries
extern crate dotenv;
extern crate pretty_env_logger;

extern crate checkout;

// Imports
use std::{thread, env};
use std::time::Duration;

use iron::prelude::*;

use checkout::create_app;
use checkout::postgres::PgDatabase;
use checkout::Database;


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

    let listener = Iron::new(create_app(
        postgres_database()
    )).http("0.0.0.0:3000").unwrap();

    println!("Server started on 0.0.0.0:3000");

    drop(listener);
}
