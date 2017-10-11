#![allow(non_snake_case)]

use iron::prelude::*;
use mount::Mount;
use uuid::Uuid;
use juniper_iron::{GraphiQLHandler, GraphQLHandler};
use juniper::{Value, Context, InputValue, FieldResult};
use serde_json;
use serde::{Serialize, Deserialize, self};

use api::*;
use schema::*;
use database::middleware::{DatabaseRequestExt, DatabaseWrapper};

struct Query;
struct Mutation;

impl Context for DatabaseWrapper {}

struct EmailRecipient(Recipient);
struct SmsRecipient(Recipient);

// Convert between GraphQL "value" and JSON "value"
fn into_scalar<T: Serialize>(v: T) -> Result<Value, serde_json::Error> {
    use serde_json::Value as JsonValue;
    use std::i32;
    fn inner(v: JsonValue) -> Value {
        match v {
            JsonValue::Null => Value::Null,
            JsonValue::Bool(b) => Value::Boolean(b),
            JsonValue::Number(n) => match (n.as_i64(), n.as_f64()) {
                (Some(v), _) if v >= i32::MIN as i64 && v <= i32::MAX as i64 => Value::Int(v as i32),
                (_, Some(v)) => Value::Float(v),
                _ => unreachable!("`as_f64()` should always return Some(value)!")
            },
            JsonValue::String(s) => Value::String(s),
            JsonValue::Array(a) => Value::List(a.into_iter().map(inner).collect()),
            JsonValue::Object(o) => Value::Object(o.into_iter().map(|(k, v)| (k, inner(v))).collect())
        }
    }

    serde_json::to_value(v).map(inner)
}

fn from_scalar<T: for<'a> Deserialize<'a>>(v: InputValue) -> Result<T, serde_json::Error> {
    use serde_json::Value as JsonValue;
    fn inner(v: InputValue) -> Result<JsonValue, ()> {
        Ok(match v {
            InputValue::Null => JsonValue::Null,
            InputValue::Int(n) => n.into(),
            InputValue::Float(n) => n.into(),
            InputValue::String(s) => JsonValue::String(s),
            InputValue::Boolean(b) => JsonValue::Bool(b),
            InputValue::List(a) => JsonValue::Array(a.into_iter().map(|s| inner(s.item)).collect::<Result<_, _>>()?),
            InputValue::Object(o) => JsonValue::Object(o.into_iter().map(|(k, v)| {
                let k = k.item;
                inner(v.item).map(|x| (k, x))
            }).collect::<Result<_, _>>()?),
            _ => return Err(())
        })
    }

    inner(v).map_err(|_| serde::de::Error::custom("Non-JSON input value")).and_then(serde_json::from_value)
}

graphql_enum!(TaskType {
    TaskType::IndividualVerifyIdentity => "INDIVIDUAL_VERIFY_IDENTITY",
    TaskType::IndividualVerifyAddress => "INDIVIDUAL_VERIFY_ADDRESS",
    TaskType::IndividualVerifySourceOfFunds => "INDIVIDUAL_VERIFY_SOURCE_OF_FUNDS",

    TaskType::IndividualAssessPoliticalExposure => "INDIVIDUAL_ASSESS_POLITICAL_EXPOSURE",
    TaskType::IndividualAssessSanctionsExposure => "INDIVIDUAL_ASSESS_SANCTIONS_EXPOSURE",
    TaskType::IndividualAssessMediaExposure => "INDIVIDUAL_ASSESS_MEDIA_EXPOSURE",
    TaskType::IndividualAssessRegulatoryStatus => "INDIVIDUAL_ASSESS_REGULATORY_STATUS",

    TaskType::CompanyVerifyIdentity => "COMPANY_VERIFY_IDENTITY",

    TaskType::CompanyIdentifyAuthorizedPersons => "COMPANY_IDENTIFY_AUTHORIZED_PERSONS",
    TaskType::CompanyIdentifyOfficers => "COMPANY_IDENTIFY_OFFICERS",
    TaskType::CompanyIdentifyBeneficialOwners => "COMPANY_IDENTIFY_BENEFICIAL_OWNERS",

    TaskType::CompanyReviewFilings => "COMPANY_REVIEW_FILINGS",

    TaskType::CompanyAssessPoliticalExposure => "COMPANY_ASSESS_POLITICAL_EXPOSURE",
    TaskType::CompanyAssessSanctionsExposure => "COMPANY_ASSESS_SANCTIONS_EXPOSURE",
});

graphql_enum!(CheckType {
    CheckType::IdentityCheck => "IDENTITY_CHECK",
    CheckType::DocumentVerification => "DOCUMENT_VERIFICATION",
    CheckType::DocumentFetch => "DOCUMENT_FETCH",
    CheckType::PepsScreen => "PEPS_SCREEN",
    CheckType::SanctionsScreen => "SANCTIONS_SCREEN",
    CheckType::PepsAndSanctionsScreen => "PEPS_AND_SANCTIONS_SCREEN",
    CheckType::AdverseMediaScreen => "ADVERSE_MEDIA_SCREEN",
    CheckType::CompanyRegistry => "COMPANY_REGISTRY",
    CheckType::CompanyOwnership => "COMPANY_OWNERSHIP",
    CheckType::CompanyFilings => "COMPANY_FILINGS",
    CheckType::CompanyFilingPurchase => "COMPANY_FILING_PURCHASE",
    CheckType::CompanyPepsAndSanctionsScreen => "COMPANY_PEPS_AND_SANCTIONS_SCREEN",
});

graphql_object!(Check: DatabaseWrapper |&self| {
    description: "A single check to run"

    field id(&executor) -> Uuid {
        self.id
    }
    field task(&executor) -> TaskType {
        self.task
    }
    field check(&executor) -> CheckType {
        self.check
    }
});

graphql_object!(Profile: DatabaseWrapper |&self| {
    description: "A profile to check"

    field id(&executor) -> Uuid {
        self.id
    }
    field possibleRecipients(&executor) -> &[Uuid] {
        &self.possible_recipients
    }
    field checks(&executor) -> &[Check] {
        &self.checks
    }
    field selectedRecipient(&executor) -> Option<Uuid> {
        self.selected_recipient
    }
    field needsInformation(&executor) -> bool {
        !self.calculated_collection_steps.is_empty()
    }
});

graphql_scalar!(PrivateArgs {
    description: "An opaque JSON object passed through to templating service"

    resolve(&self) -> Value {
        into_scalar(self).expect("Failed to serialize private args")
    }

    from_input_value(v: &InputValue) -> Option<PrivateArgs> {
        from_scalar(v.clone()).ok()
    }
});

graphql_object!(PublicArgs: DatabaseWrapper |&self| {
    description: "The set of fields customisable by the front-end"

    field from(&executor) -> &Option<String> {
        &self.from
    }
    field bcc(&executor) -> &Vec<String> {
        &self.bcc
    }
});

graphql_object!(Communication: DatabaseWrapper |&self| {
    description: "A communication with the end user"

    field recipient(&executor) -> Uuid {
        self.recipient
    }
    field publicArgs(&executor) -> &PublicArgs {
        &self.public_args
    }
    field privateArgs(&executor) -> &PrivateArgs {
        &self.private_args
    }
});

graphql_object!(EmailRecipient: DatabaseWrapper |&self| {
    description: "A recipient to be contacted via email"

    field id(&executor) -> Uuid {
        self.0.id
    }
    field name(&executor) -> &str {
        &self.0.name
    }
    field address(&executor) -> &str {
        if let ContactMethod::Email { ref address } = self.0.contact_method {
            address
        } else {
            unreachable!()
        }
    }

    interfaces: [Recipient]
});

graphql_object!(SmsRecipient: DatabaseWrapper |&self| {
    description: "A recipient to be contacted via SMS"

    field id(&executor) -> Uuid {
        self.0.id
    }
    field name(&executor) -> &str {
        &self.0.name
    }
    field phoneNumber(&executor) -> &str {
        if let ContactMethod::Sms { ref phone_number } = self.0.contact_method {
            phone_number
        } else {
            unreachable!()
        }
    }

    interfaces: [Recipient]
});

graphql_interface!(Recipient: DatabaseWrapper |&self| {
    description: "A recipient of a communication"

    field id(&executor) -> Uuid {
        self.id
    }
    field name(&executor) -> &str {
        &self.name
    }
    instance_resolvers: |_| {
        EmailRecipient => if let ContactMethod::Email {..} = self.contact_method { Some(EmailRecipient(self.clone())) } else { None },
        SmsRecipient => if let ContactMethod::Sms {..} = self.contact_method { Some(SmsRecipient(self.clone())) } else { None },
    }
});

graphql_object!(Basket: DatabaseWrapper |&self| {
    description: "A single basket"

    field id(&executor) -> Uuid {
        self.id
    }
    field profilesToCheck(&executor) -> &[Profile] {
        &self.contents.0.profiles_to_check
    }
    field communications(&executor) -> &[Communication] {
        &self.contents.0.communications
    }
    field recipients(&executor) -> &[Recipient] {
        &self.contents.0.recipients
    }
});

graphql_object!(Query: DatabaseWrapper |&self| {
    description: "The root query object of the schema"
    
    field basket(&executor, id: Uuid) -> FieldResult<Basket> {
        executor.context().update_basket(id, &mut |_| Ok(()))
    }
});

graphql_object!(Mutation: DatabaseWrapper |&self| {
    description: "The root mutation object of the schema"

    field setRecipientOnProfile(&executor, basketId: Uuid, profileId: Uuid, recipientId: Option<Uuid>) -> FieldResult<Basket> {
        executor.context().update_basket(basketId, &mut |basket| {
            let profile = basket.contents.0.find_profile_mut(profileId).ok_or("Profile ID not found")?;
            profile.selected_recipient = recipientId;
            Ok(())
        })
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
