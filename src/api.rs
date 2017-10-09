use std::collections::BTreeSet;
use uuid::Uuid;


#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskType {
    IndividualVerifyIdentity,
    IndividualVerifyAddress,
    IndividualVerifySourceOfFunds,

    IndividualAssessPoliticalExposure,
    IndividualAssessSanctionsExposure,
    IndividualAssessMediaExposure,
    IndividualAssessRegulatoryStatus,

    CompanyVerifyIdentity,

    CompanyIdentifyAuthorizedPersons,
    CompanyIdentifyOfficers,
    CompanyIdentifyBeneficialOwners,

    CompanyReviewFilings,

    CompanyAssessPoliticalExposure,
    CompanyAssessSanctionsExposure,
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CheckType {
    IdentityCheck,
    DocumentVerification,
    DocumentFetch,
    PepsScreen,
    SanctionsScreen,
    PepsAndSanctionsScreen,
    AdverseMediaScreen,
    CompanyRegistry,
    CompanyOwnership,
    CompanyFilings,
    CompanyFilingPurchase,
    CompanyPepsAndSanctionsScreen,
}


#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DatePrecision {
    Year,
    YearMonth,
    YearMonthDay
}


#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocumentCategory {
    ProofOfIdentity,
    ProofOfAddress,
    Supporting,
    CompanyFiling,
    DataSummary
}


#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocumentType {
    Passport,
    DrivingLicence,
    StateId,
    BirthCertificate,
    BankStatement,
    FaceImage,
    Unknown,
    CompanyAccounts,
    CompanyChangeOfAddress,
    AnnualReturn,
    ConfirmationStatement,
    StatementOfCaptital,
    ChangeOfName,
    Incorporation,
    Liquidation,
    Miscellaneous,
    Mortgage,
    ChangeOfOfficers,
    Resolution,
    CreditReport,
    CreditCheck,
    RegisterReport,
    RegisterCheck,
    DataSummary
}


#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CollectionStep {
    FullName {},
    Dob {
        precision: DatePrecision
    },
    AddressHistory {
        months: u32
    },
    Nationality {},
    Document {
        id: Uuid,
        category: DocumentCategory,
        allowed_types: BTreeSet<DocumentType>
    }
}

impl CollectionStep {
    pub fn try_merge(&mut self, other: &CollectionStep) -> bool {
        use self::CollectionStep::*;
        match (self, other) {
            (&mut FullName {}, &FullName {}) => true,
            (&mut Dob { ref mut precision }, &Dob { precision: other_precision }) => {
                if other_precision > *precision {
                    *precision = other_precision;
                }
                true
            },
            (&mut AddressHistory { ref mut months }, &AddressHistory { months: other_months }) => {
                if other_months > *months {
                    *months = other_months;
                }
                true
            },
            (&mut Nationality {}, &Nationality {}) => true,
            (&mut Document {id, ref mut allowed_types, ..}, &Document {id: other_id, allowed_types: ref other_allowed_types, ..}) if id == other_id => {
                *allowed_types = &*allowed_types & other_allowed_types;
                true
            },
            _ => false
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct FullName {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_family_names: Option<Vec<String>>
}


#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct PersonalDetails {
    name: FullName,
    #[serde(skip_serializing_if = "Option::is_none")]
    dob: Option<PartialDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nationality: Option<String>
}

pub type ApproxDate = String;
pub type PartialDate = String;


#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct StructuredAddress {
    pub country: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_province: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub county: Option<String>,
    pub postal_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postal_town: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub street_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub premise: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subpremise: Option<String>,
}


#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct FreeformAddress {
    country: String,
    text: String
}


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Address {
    StructuredAddress(StructuredAddress),
    FreeformAddress(FreeformAddress)
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatedAddress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<ApproxDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<ApproxDate>,
    pub address: Address
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Document {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub category: DocumentCategory,
    pub document_type: DocumentType
}


#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct IndividualData {
    personal_details: PersonalDetails,
    #[serde(skip_serializing_if = "Option::is_none")]
    address_history: Option<Vec<DatedAddress>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    documents: Option<Vec<Document>>
}


#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct CompanyData {
}


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "entity_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntityData {
    IndividualData(IndividualData),
    CompanyData(CompanyData)
}
