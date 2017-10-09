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
    pub checks: Vec<EntityCheck>,
    pub manual_collection_steps: Vec<CollectionStep>
}

fn merge_collection_steps(into: &mut Vec<CollectionStep>, src: &[CollectionStep]) {
    'next_step: for collection_step in src {
        for item in into.iter_mut() {
            if item.try_merge(&collection_step) {
                continue 'next_step;
            }
        }
        into.push(collection_step.clone());
    }
}

impl EntityChecks {
    pub fn calculate_collection_steps(&self) -> Vec<CollectionStep> {
        let mut result: Vec<CollectionStep> = Vec::new();

        // Add any collection steps generated from checks
        for check in &self.checks {
            merge_collection_steps(&mut result, &check.calculate_collection_steps());
        }

        // Add any manually added collection steps
        merge_collection_steps(&mut result, &self.manual_collection_steps);

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn merge_address_collection_steps() {
        let checks = EntityChecks {
            checks: vec![
                EntityCheck {
                    task: TaskType::IndividualVerifyIdentity,
                    check: CheckType::IdentityCheck,
                    contact: None
                }
            ],
            manual_collection_steps: vec![
                CollectionStep::AddressHistory {
                    months: 3
                }
            ]
        };

        let actual_collection_steps: HashSet<_> = checks.calculate_collection_steps().into_iter().collect();
        let expected_collection_steps: HashSet<_> = vec![
            CollectionStep::FullName {},
            CollectionStep::Dob { precision: DatePrecision::YearMonthDay },
            CollectionStep::AddressHistory { months: 3 }
        ].into_iter().collect();

        assert_eq!(actual_collection_steps, expected_collection_steps);
    }
}
