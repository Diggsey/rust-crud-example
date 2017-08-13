use std::sync::Arc;

use iron::prelude::*;
use iron::{typemap, BeforeMiddleware};

use database::interface::Database;

// The DatabaseMiddleware is constructed from a concrete
// database implementation, and provides it to all
// request handlers via the `.db()` method.

// Shared reference to a database implementation
pub struct DatabaseMiddleware(Arc<Database>);

impl DatabaseMiddleware {
    pub fn new<T: Database>(db: T) -> Self {
        DatabaseMiddleware(Arc::new(db))
    }
}

// Tell Iron what kind of value we want to attach to each request.
impl typemap::Key for DatabaseMiddleware { type Value = Arc<Database>; }

// Before each request, attach a reference to the database.
impl BeforeMiddleware for DatabaseMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<Self>(self.0.clone());
        Ok(())
    }
}

// Add an extension method to Request objects to access the database.
pub trait DatabaseRequestExt {
    fn db(&self) -> &Database;
}
impl<'a, 'b> DatabaseRequestExt for Request<'a, 'b> {
    fn db(&self) -> &Database {
        &**self.extensions.get::<DatabaseMiddleware>()
            .expect("DatabaseMiddleware not registered")
    }
}
