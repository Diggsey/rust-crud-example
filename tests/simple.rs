extern crate checkout;
extern crate iron_test;
extern crate iron;

use iron_test::request;
use iron_test::response::extract_body_to_string;
use iron::{Headers, Handler};
use iron::status::Status;

use checkout::{Database, create_app, schema};

#[derive(Debug)]
struct MockDatabase;

impl Database for MockDatabase {
    fn list_baskets(&self) -> Vec<schema::Basket> { Vec::new() }
}

fn get<H: Handler>(url: &str, app: &H) -> (Status, String) {
    let response = request::get(&format!("http://localhost:3000{}", url), Headers::new(), app).unwrap();
    (
        response.status.unwrap(),
        extract_body_to_string(response)
    )
}

fn post<H: Handler>(url: &str, app: &H, content: &str) -> (Status, String) {
    let response = request::post(&format!("http://localhost:3000{}", url), Headers::new(), content, app).unwrap();
    (
        response.status.unwrap(),
        extract_body_to_string(response)
    )
}

#[test]
fn graphiql_test() {
    // Verify that we return the GraphiQL interface
    let app = create_app(MockDatabase);
    let (code, response) = get("/", &app);
    assert_eq!(code, Status::Ok);
    assert!(response.trim_left().starts_with("<!DOCTYPE html>"));
}

#[test]
fn smoke_test() {
    // Verify that we can run a query
    let app = create_app(MockDatabase);
    let (code, response) = post("/graphql", &app, r#"{
        "query": "{ baskets { id } }"
    }"#);
    assert_eq!(code, Status::Ok);
    assert_eq!(response, 
r#"{
  "data": {
    "baskets": []
  }
}"#
    );
}
