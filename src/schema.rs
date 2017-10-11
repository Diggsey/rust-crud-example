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
pub struct Check {
    pub id: Uuid,
    pub task: TaskType,
    pub check: CheckType,
}


impl Check {
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
pub struct Profile {
    pub id: Uuid,
    pub possible_recipients: Vec<Uuid>,
    pub checks: Vec<Check>,
    pub selected_recipient: Option<Uuid>,
    pub extra_collection_steps: Vec<CollectionStep>,
    pub calculated_collection_steps: Vec<CollectionStep>
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

impl Profile {
    pub fn recalculate_collection_steps(&mut self) {
        // Clear collection steps
        self.calculated_collection_steps.clear();

        // Add any collection steps generated from checks
        for check in &self.checks {
            merge_collection_steps(&mut self.calculated_collection_steps, &check.calculate_collection_steps());
        }

        // Add any extra collection steps
        merge_collection_steps(&mut self.calculated_collection_steps, &self.extra_collection_steps);
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PublicArgs {
    pub from: Option<String>,
    pub bcc: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PrivateArgs {
    pub from: Option<String>,
    pub bcc: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Communication {
    pub recipient: Uuid,
    pub public_args: PublicArgs,
    pub private_args: PrivateArgs,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ContactMethod {
    Email {
        address: String
    },
    Sms {
        phone_number: String
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Recipient {
    pub id: Uuid,
    pub name: String,
    pub contact_method: ContactMethod,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BasketContentsV1 {
    pub profiles_to_check: Vec<Profile>,
    pub communications: Vec<Communication>,
    pub recipients: Vec<Recipient>
}

impl BasketContentsV1 {
    pub fn find_profile_mut(&mut self, profile_id: Uuid) -> Option<&mut Profile> {
        self.profiles_to_check.iter_mut()
            .filter(|p| p.id == profile_id)
            .next()
    }
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
        let mut profile = Profile {
            checks: vec![
                Check {
                    id: Default::default(),
                    task: TaskType::IndividualVerifyIdentity,
                    check: CheckType::IdentityCheck,
                }
            ],
            extra_collection_steps: vec![
                CollectionStep::AddressHistory { months: 3 }
            ],
            ..Default::default()
        };

        profile.recalculate_collection_steps();

        let actual_collection_steps: HashSet<_> = profile.calculated_collection_steps.into_iter().collect();
        let expected_collection_steps: HashSet<_> = vec![
            CollectionStep::FullName {},
            CollectionStep::Dob { precision: DatePrecision::YearMonthDay },
            CollectionStep::AddressHistory { months: 3 }
        ].into_iter().collect();

        assert_eq!(actual_collection_steps, expected_collection_steps);
    }
}
