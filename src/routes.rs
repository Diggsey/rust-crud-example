use iron::prelude::*;
use iron::status;
use router::Router;
use uuid::Uuid;

use schema::*;
use database::middleware::DatabaseRequestExt;

pub fn get() -> Router {
    let mut router = Router::new();
    router.get("/", index, "index");
    router.get("/new", add_basket, "add_basket");
    router.get("/list", list_baskets, "list_baskets");
    router
}

fn index(req: &mut Request) -> IronResult<Response> {
    let ref query = req.extensions.get::<Router>().unwrap().find("query").unwrap_or("/");
    Ok(Response::with((status::Ok, *query)))
}

fn add_basket(req: &mut Request) -> IronResult<Response> {
    let basket = Basket {
        id: Uuid::new_v4(),
        contents: Default::default()
    };
    req.db().add_basket(basket);
    Ok(Response::with((status::Ok, "Done")))
}

fn list_baskets(req: &mut Request) -> IronResult<Response> {
    let result = req.db().list_baskets();
    Ok(Response::with((status::Ok, format!("{:?}", result))))
}
