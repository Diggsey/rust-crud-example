use schema::*;

// Database must:
// - be thread-safe (Send + Sync)
// - live as long as required ('static)
pub trait Database: Send + Sync + 'static {
    fn list_baskets(&self) -> Vec<Basket> { unimplemented!() }
    fn add_basket(&self, _basket: Basket) { unimplemented!() }
    fn migrate(&self) { unimplemented!() }
}
