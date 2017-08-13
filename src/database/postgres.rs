use std::error::Error;

use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::result::QueryResult;
use r2d2;
use r2d2_diesel::ConnectionManager;

use schema::*;
use database::interface::Database;

// Implement a postgres database backend using a connection pool
pub struct PgDatabase(r2d2::Pool<ConnectionManager<PgConnection>>);

impl PgDatabase {
    // Connect to the database
    pub fn connect(connection_str: &str) -> Result<PgDatabase, Box<Error>> {
        let config = r2d2::Config::default();
        let manager = ConnectionManager::new(connection_str);
        let pool = try!(r2d2::Pool::new(config, manager));
        Ok(PgDatabase(pool))
    }

    // Run some code in a transaction, and retry it automatically
    // if the transaction is rolled back.
    fn execute<R, F>(&self, mut f: F) -> R where F: FnMut(&PgConnection) -> QueryResult<R> {
        use std::thread;
        use std::time::Duration;

        // Get a connection from the pool
        let conn = self.0.get()
            .expect("Failed to obtain database connection");

        let mut num_attempts = 0;
        loop {
            // Try running the code in a transaction
            match conn.transaction(|| f(&conn)) {
                // Success, return result
                Ok(r) => break r,
                // Error, retry 5 times with backoff
                Err(_) if num_attempts < 5 => {
                    thread::sleep(Duration::from_millis(10 << num_attempts));
                    num_attempts += 1;
                },
                // Any other error, panic
                Err(e) => panic!("Database error: {}", e)
            }
        }
    }
}

// Implement all the operations supported by the database
impl Database for PgDatabase {
    // fn list_baskets(&self) -> Vec<Basket> {
    //     self.execute(|conn| {
    //         baskets::table.load::<Basket>(conn)
    //     })
    // }
    fn add_basket(&self, basket: Basket) {
        self.execute(|conn| {
            try!(diesel::insert(&basket).into(baskets::table)
                .get_result::<Basket>(conn));
            Ok(())
        })
    }
}
