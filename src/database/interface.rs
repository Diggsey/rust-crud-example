use std::fmt::Debug;
use schema::*;
use uuid::Uuid;

// Database must:
// - be thread-safe (Send + Sync)
// - live as long as required ('static)
pub trait Database: Send + Sync + 'static + Debug {
    fn update_basket_impl(&self, _basket_id: Uuid, _f: &mut FnMut(&mut Basket)) -> Basket { unimplemented!() }
    fn migrate(&self) { unimplemented!() }
}

impl Database {
    pub fn update_basket<E, F: FnMut(&mut Basket) -> Result<(), E>>(&self, basket_id: Uuid, f: &mut F) -> Result<Basket, E> {
        let mut result = None;
        let basket = self.update_basket_impl(basket_id, &mut |basket| {
            result = Some(f(basket));
        });
        result.expect("Failed to execute callback!").map(|_| basket)
    }
}
