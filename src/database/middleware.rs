use std::sync::Arc;
use std::ops::Deref;

use iron::prelude::*;
use iron::{typemap, BeforeMiddleware};

use database::interface::Database;

// The DatabaseMiddleware is constructed from a concrete
// database implementation, and provides it to all
// request handlers via the `.db()` method.

// Shared reference to a database implementation
#[derive(Clone, Debug)]
pub struct DatabaseWrapper(Arc<Database>);

impl DatabaseWrapper {
    pub fn new<T: Database>(db: T) -> Self {
        DatabaseWrapper(Arc::new(db))
    }
}

impl Deref for DatabaseWrapper {
    type Target = Database;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

// Tell Iron what kind of value we want to attach to each request.
impl typemap::Key for DatabaseWrapper { type Value = Self; }

// Before each request, attach a reference to the database.
impl BeforeMiddleware for DatabaseWrapper {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<Self>(self.clone());
        Ok(())
    }
}

// Add an extension method to Request objects to access the database.
pub trait DatabaseRequestExt {
    fn db(&self) -> DatabaseWrapper;
}

impl<'a, 'b> DatabaseRequestExt for Request<'a, 'b> {
    fn db(&self) -> DatabaseWrapper {
        self.extensions.get::<DatabaseWrapper>()
            .expect("DatabaseMiddleware not registered")
            .clone()
    }
}
