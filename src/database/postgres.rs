use std::error::Error;
use std::io;
use std::fmt;

use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::result::QueryResult;
use r2d2;
use r2d2_diesel::ConnectionManager;
use uuid::Uuid;

use schema::*;
use database::interface::Database;

embed_migrations!("migrations");

// Implement a postgres database backend using a connection pool
pub struct PgDatabase(r2d2::Pool<ConnectionManager<PgConnection>>);

impl fmt::Debug for PgDatabase {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "PgDatabase")
    }
}

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
    fn update_basket_impl(&self, basket_id: Uuid, f: &mut FnMut(&mut Basket)) -> Basket {
        self.execute(|conn| {
            // Find an existing basket if one exists
            let maybe_basket = baskets::table.find(basket_id).first::<Basket>(conn);
            let is_new_basket = maybe_basket.is_err();

            // Create a new basket if none exists
            let mut basket = maybe_basket.unwrap_or_else(|_| {
                Basket {
                    id: basket_id,
                    contents: Default::default()
                }
            });

            // Run the update on the basket
            f(&mut basket);

            // Update the database  
            if is_new_basket {
                basket = diesel::insert(&basket).into(baskets::table)
                    .get_result::<Basket>(conn)?;
            } else {
                basket = diesel::update(baskets::table.find(basket_id))
                    .set(baskets::contents.eq(basket.contents))
                    .get_result::<Basket>(conn)?;
            }

            // Return the updated basket
            Ok(basket)
        })
    }
    fn migrate(&self) {
        let conn = self.0.get()
            .expect("Failed to obtain database connection");
        embedded_migrations::run_with_output(&*conn, &mut io::stdout())
            .expect("Failed to run migration");
    }
}
