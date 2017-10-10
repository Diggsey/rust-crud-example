extern crate checkout;
extern crate iron_test;
extern crate iron;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

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

fn test_query<H: Handler>(app: &H, query: &str, expected_response: &str) {
    #[derive(Serialize)]
    struct GraphQlRequest<'a> {
        query: &'a str
    }

    let (code, response) = post("/graphql", app, &serde_json::to_string(&GraphQlRequest {
        query
    }).unwrap());

    assert_eq!(code, Status::Ok);

    let response_value = serde_json::from_str::<serde_json::Value>(&response).unwrap();
    let expected_value = serde_json::from_str::<serde_json::Value>(expected_response).unwrap();
    assert_eq!(response_value, expected_value);
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
    test_query(&app, "{ baskets { id } }", r#"{
        "data": {
            "baskets": []
        }
    }"#);
}
