//! HL7 FHIR R5 API implementation

use crate::models::Patient;
use crate::Result;

pub mod resources;
pub mod bundle;
pub mod search_parameters;

/// Convert internal Patient model to FHIR Patient resource
pub fn to_fhir_patient(patient: &Patient) -> serde_json::Value {
    // TODO: Implement FHIR Patient resource mapping
    serde_json::json!({
        "resourceType": "Patient",
        "id": patient.id.to_string(),
        // ... additional FHIR fields
    })
}

/// Convert FHIR Patient resource to internal Patient model
pub fn from_fhir_patient(fhir_patient: &serde_json::Value) -> Result<Patient> {
    // TODO: Implement FHIR Patient resource parsing
    todo!("Implement FHIR Patient resource parsing")
}
