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
mod api;
pub mod schema;
mod routes;
mod database;

use iron::prelude::*;
use logger::Logger;

use database::middleware::DatabaseWrapper;

pub use database::interface::Database;
pub use database::postgres;

// Inject dependencies and return an application
pub fn create_app<D: Database>(
    db: D
) -> Chain {
    let mut chain = Chain::new(routes::get());
    chain.link(Logger::new(None));
    chain.link_before(DatabaseWrapper::new(db));
    chain
}
