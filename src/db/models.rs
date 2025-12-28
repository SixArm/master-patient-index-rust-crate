//! Database models (Diesel ORM entities)
//!
//! These models are used for database operations and are separate from
//! the domain models in src/models to maintain separation of concerns.

use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::schema::*;

// ============================================================================
// Patient Models
// ============================================================================

/// Patient database model (Queryable)
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = patients)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbPatient {
    pub id: Uuid,
    pub active: bool,
    pub gender: String,
    pub birth_date: Option<NaiveDate>,
    pub deceased: bool,
    pub deceased_datetime: Option<DateTime<Utc>>,
    pub marital_status: Option<String>,
    pub multiple_birth: Option<bool>,
    pub managing_organization_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<String>,
}

/// New patient model (Insertable)
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = patients)]
pub struct NewDbPatient {
    pub id: Option<Uuid>,
    pub active: bool,
    pub gender: String,
    pub birth_date: Option<NaiveDate>,
    pub deceased: bool,
    pub deceased_datetime: Option<DateTime<Utc>>,
    pub marital_status: Option<String>,
    pub multiple_birth: Option<bool>,
    pub managing_organization_id: Option<Uuid>,
    pub created_by: Option<String>,
}

/// Patient update model
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = patients)]
pub struct UpdateDbPatient {
    pub active: Option<bool>,
    pub gender: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub deceased: Option<bool>,
    pub deceased_datetime: Option<DateTime<Utc>>,
    pub marital_status: Option<String>,
    pub multiple_birth: Option<bool>,
    pub managing_organization_id: Option<Uuid>,
    pub updated_by: Option<String>,
}

// ============================================================================
// Patient Name Models
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = patient_names)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbPatientName {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub use_type: Option<String>,
    pub family: String,
    pub given: Vec<String>,
    pub prefix: Vec<String>,
    pub suffix: Vec<String>,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = patient_names)]
pub struct NewDbPatientName {
    pub patient_id: Uuid,
    pub use_type: Option<String>,
    pub family: String,
    pub given: Vec<String>,
    pub prefix: Vec<String>,
    pub suffix: Vec<String>,
    pub is_primary: bool,
}

// ============================================================================
// Patient Identifier Models
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = patient_identifiers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbPatientIdentifier {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub use_type: Option<String>,
    pub identifier_type: String,
    pub system: String,
    pub value: String,
    pub assigner: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = patient_identifiers)]
pub struct NewDbPatientIdentifier {
    pub patient_id: Uuid,
    pub use_type: Option<String>,
    pub identifier_type: String,
    pub system: String,
    pub value: String,
    pub assigner: Option<String>,
}

// ============================================================================
// Patient Address Models
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = patient_addresses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbPatientAddress {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub use_type: Option<String>,
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = patient_addresses)]
pub struct NewDbPatientAddress {
    pub patient_id: Uuid,
    pub use_type: Option<String>,
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub is_primary: bool,
}

// ============================================================================
// Patient Contact Models
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = patient_contacts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbPatientContact {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub system: String,
    pub value: String,
    pub use_type: Option<String>,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = patient_contacts)]
pub struct NewDbPatientContact {
    pub patient_id: Uuid,
    pub system: String,
    pub value: String,
    pub use_type: Option<String>,
    pub is_primary: bool,
}

// ============================================================================
// Patient Link Models
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = patient_links)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbPatientLink {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub other_patient_id: Uuid,
    pub link_type: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = patient_links)]
pub struct NewDbPatientLink {
    pub patient_id: Uuid,
    pub other_patient_id: Uuid,
    pub link_type: String,
    pub created_by: Option<String>,
}

// ============================================================================
// Organization Models
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = organizations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbOrganization {
    pub id: Uuid,
    pub active: bool,
    pub name: String,
    pub alias: Vec<String>,
    pub org_type: Vec<String>,
    pub part_of: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<String>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = organizations)]
pub struct NewDbOrganization {
    pub id: Option<Uuid>,
    pub active: bool,
    pub name: String,
    pub alias: Vec<String>,
    pub org_type: Vec<String>,
    pub part_of: Option<Uuid>,
    pub created_by: Option<String>,
}

// ============================================================================
// Patient Match Score Models
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = patient_match_scores)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbPatientMatchScore {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub candidate_id: Uuid,
    pub total_score: bigdecimal::BigDecimal,
    pub name_score: Option<bigdecimal::BigDecimal>,
    pub birth_date_score: Option<bigdecimal::BigDecimal>,
    pub gender_score: Option<bigdecimal::BigDecimal>,
    pub address_score: Option<bigdecimal::BigDecimal>,
    pub identifier_score: Option<bigdecimal::BigDecimal>,
    pub calculated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = patient_match_scores)]
pub struct NewDbPatientMatchScore {
    pub patient_id: Uuid,
    pub candidate_id: Uuid,
    pub total_score: bigdecimal::BigDecimal,
    pub name_score: Option<bigdecimal::BigDecimal>,
    pub birth_date_score: Option<bigdecimal::BigDecimal>,
    pub gender_score: Option<bigdecimal::BigDecimal>,
    pub address_score: Option<bigdecimal::BigDecimal>,
    pub identifier_score: Option<bigdecimal::BigDecimal>,
}

// ============================================================================
// Audit Log Models
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = audit_log)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DbAuditLog {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = audit_log)]
pub struct NewDbAuditLog {
    pub user_id: Option<String>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
