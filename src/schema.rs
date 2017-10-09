use std::collections::HashMap;
use uuid::Uuid;
use serde_json;

use api::{TaskType, CheckType, CollectionStep, DatePrecision};


table! {
    baskets (id) {
        id -> Uuid,
        contents -> Jsonb,
    }
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ContactMethod {
    Email {
        address: String,
        subject: String,
        cover_note: String
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EntityCheck {
    pub task: TaskType,
    pub check: CheckType,
    pub contact: Option<ContactMethod>
}


impl EntityCheck {
    pub fn calculate_collection_steps(&self) -> Vec<CollectionStep> {
        let mut result = Vec::new();
        match self.check {
            CheckType::IdentityCheck => {
                result.push(CollectionStep::FullName {});
                result.push(CollectionStep::Dob { precision: DatePrecision::YearMonthDay });
                result.push(CollectionStep::AddressHistory { months: 0 });
            },
            CheckType::PepsAndSanctionsScreen | CheckType::PepsScreen | CheckType::SanctionsScreen => {
                result.push(CollectionStep::FullName {});
                result.push(CollectionStep::Dob { precision: DatePrecision::YearMonthDay });
                result.push(CollectionStep::Nationality {});
            },
            CheckType::DocumentVerification => {
                result.push(CollectionStep::Nationality {});
            },
            _ => unimplemented!()
        }
        result
    }
}


#[derive(Serialize, Deserialize, Debug, Default)]
pub struct EntityChecks {
    pub checks: Vec<EntityCheck>
}

impl EntityChecks {
    pub fn calculate_collection_steps(&self) -> Vec<CollectionStep> {
        let mut result: Vec<CollectionStep> = Vec::new();
        for check in &self.checks {
            'next_step: for collection_step in check.calculate_collection_steps() {
                for item in &mut result {
                    if item.try_merge(&collection_step) {
                        continue 'next_step;
                    }
                }
                result.push(collection_step);
            }
        }
        result
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BasketContentsV1 {
    pub entities: HashMap<Uuid, EntityChecks>
}

version_json_type!(
    #[derive(Debug, Default)]
    basket_contents BasketContents {
        V1 => BasketContentsV1 {}
    }
);

#[derive(Queryable, Insertable, Default, Debug)]
#[table_name="baskets"]
pub struct Basket {
    pub id: Uuid,
    pub contents: BasketContents
}
