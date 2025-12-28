//! Repository pattern implementations for database operations

use crate::models::Patient;
use crate::Result;

/// Patient repository trait
pub trait PatientRepository {
    /// Create a new patient
    fn create(&self, patient: &Patient) -> Result<Patient>;

    /// Get a patient by ID
    fn get_by_id(&self, id: &uuid::Uuid) -> Result<Option<Patient>>;

    /// Update a patient
    fn update(&self, patient: &Patient) -> Result<Patient>;

    /// Delete a patient (soft delete)
    fn delete(&self, id: &uuid::Uuid) -> Result<()>;

    /// Search patients
    fn search(&self, query: &str) -> Result<Vec<Patient>>;
}
