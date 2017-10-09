use iron::prelude::*;
use mount::Mount;
use uuid::Uuid;
use juniper::iron_handlers::{GraphiQLHandler, GraphQLHandler};
use juniper::{Value, Context};

use api::*;
use schema::*;
use database::middleware::{DatabaseRequestExt, DatabaseWrapper};

struct Query;
struct Mutation;

struct UuidWrapper(Uuid);

impl Context for DatabaseWrapper {}

graphql_scalar!(UuidWrapper as "Uuid" {
    description: "A UUID"

    resolve(&self) -> Value {
        Value::string(&self.0.to_string())
    }

    from_input_value(v: &InputValue) -> Option<UuidWrapper> {
        v.as_string_value().and_then(|s| Uuid::parse_str(s).ok()).map(UuidWrapper)
    }
});

graphql_object!(Basket: DatabaseWrapper |&self| {
    description: "A single basket"

    field id(&executor) -> UuidWrapper {
        UuidWrapper(self.id)
    }
});

graphql_object!(Query: DatabaseWrapper |&self| {
    description: "The root query object of the schema"

    field baskets(&executor) -> Vec<Basket> {
        executor.context().list_baskets()
    }
});

graphql_object!(Mutation: DatabaseWrapper |&self| {
    description: "The root mutation object of the schema"

    field add_basket(&executor) -> Basket {
        let mut basket = Basket {
            id: Uuid::new_v4(),
            contents: Default::default()
        };

        let mut entity_checks: EntityChecks = Default::default();
        entity_checks.checks.push(EntityCheck {
            task: TaskType::IndividualVerifyIdentity,
            check: CheckType::IdentityCheck,
            contact: None
        });

        basket.contents.0.entities.insert(basket.id, entity_checks);

        executor.context().add_basket(&basket);
        basket
    }
});

fn context_factory(req: &mut Request) -> DatabaseWrapper {
    req.db()
}

pub fn get() -> Mount {
    let mut mount = Mount::new();

    let graphql_endpoint = GraphQLHandler::new(
        context_factory,
        Query,
        Mutation,
    );
    let graphiql_endpoint = GraphiQLHandler::new("graphql");

    mount.mount("/", graphiql_endpoint);
    mount.mount("/graphql", graphql_endpoint);
    mount
}
