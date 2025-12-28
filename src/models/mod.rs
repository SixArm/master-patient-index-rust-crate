//! Data models for the MPI system

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod patient;
pub mod organization;
pub mod identifier;

pub use patient::{Patient, HumanName, NameUse, PatientLink, LinkType};
pub use organization::Organization;
pub use identifier::{Identifier, IdentifierType, IdentifierUse};

/// Gender enumeration per FHIR specification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    Male,
    Female,
    Other,
    Unknown,
}

/// Address information
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Address {
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

/// Contact information
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ContactPoint {
    pub system: ContactPointSystem,
    pub value: String,
    pub use_type: Option<ContactPointUse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ContactPointSystem {
    Phone,
    Fax,
    Email,
    Pager,
    Url,
    Sms,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ContactPointUse {
    Home,
    Work,
    Temp,
    Old,
    Mobile,
}
