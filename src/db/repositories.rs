//! Repository pattern implementations for database operations

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use chrono::Utc;
use uuid::Uuid;

use crate::models::{Patient, HumanName, Address, ContactPoint, Identifier, PatientLink};
use crate::Result;
use super::models::*;
use super::schema::*;

/// Audit context for tracking user actions
#[derive(Debug, Clone)]
pub struct AuditContext {
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl Default for AuditContext {
    fn default() -> Self {
        Self {
            user_id: Some("system".to_string()),
            ip_address: None,
            user_agent: None,
        }
    }
}

/// Patient repository trait
pub trait PatientRepository: Send + Sync {
    /// Create a new patient
    fn create(&self, patient: &Patient) -> Result<Patient>;

    /// Get a patient by ID
    fn get_by_id(&self, id: &Uuid) -> Result<Option<Patient>>;

    /// Update a patient
    fn update(&self, patient: &Patient) -> Result<Patient>;

    /// Delete a patient (soft delete)
    fn delete(&self, id: &Uuid) -> Result<()>;

    /// Search patients by name
    fn search(&self, query: &str) -> Result<Vec<Patient>>;

    /// List all active patients (non-deleted)
    fn list_active(&self, limit: i64, offset: i64) -> Result<Vec<Patient>>;
}

/// Diesel-based patient repository implementation
pub struct DieselPatientRepository {
    pool: Pool<ConnectionManager<PgConnection>>,
    event_publisher: Option<std::sync::Arc<dyn crate::streaming::EventProducer>>,
    audit_log: Option<std::sync::Arc<super::audit::AuditLogRepository>>,
}

impl DieselPatientRepository {
    /// Create a new repository with the given connection pool
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self {
            pool,
            event_publisher: None,
            audit_log: None,
        }
    }

    /// Set the event publisher for this repository
    pub fn with_event_publisher(
        mut self,
        publisher: std::sync::Arc<dyn crate::streaming::EventProducer>,
    ) -> Self {
        self.event_publisher = Some(publisher);
        self
    }

    /// Set the audit log repository
    pub fn with_audit_log(
        mut self,
        audit_log: std::sync::Arc<super::audit::AuditLogRepository>,
    ) -> Self {
        self.audit_log = Some(audit_log);
        self
    }

    /// Publish an event if publisher is configured
    fn publish_event(&self, event: crate::streaming::PatientEvent) {
        if let Some(ref publisher) = self.event_publisher {
            if let Err(e) = publisher.publish(event) {
                tracing::error!("Failed to publish event: {}", e);
            }
        }
    }

    /// Log to audit trail if configured
    fn log_audit(
        &self,
        action: &str,
        entity_id: uuid::Uuid,
        old_values: Option<serde_json::Value>,
        new_values: Option<serde_json::Value>,
        context: &AuditContext,
    ) {
        if let Some(ref audit_log) = self.audit_log {
            let result = match action {
                "CREATE" => audit_log.log_create(
                    "Patient",
                    entity_id,
                    new_values.unwrap_or(serde_json::Value::Null),
                    context.user_id.clone(),
                    context.ip_address.clone(),
                    context.user_agent.clone(),
                ),
                "UPDATE" => audit_log.log_update(
                    "Patient",
                    entity_id,
                    old_values.unwrap_or(serde_json::Value::Null),
                    new_values.unwrap_or(serde_json::Value::Null),
                    context.user_id.clone(),
                    context.ip_address.clone(),
                    context.user_agent.clone(),
                ),
                "DELETE" => audit_log.log_delete(
                    "Patient",
                    entity_id,
                    old_values.unwrap_or(serde_json::Value::Null),
                    context.user_id.clone(),
                    context.ip_address.clone(),
                    context.user_agent.clone(),
                ),
                _ => Ok(()),
            };

            if let Err(e) = result {
                tracing::error!("Failed to log audit: {}", e);
            }
        }
    }

    /// Get a database connection from the pool
    fn get_conn(&self) -> Result<diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>>> {
        self.pool.get().map_err(|e| crate::Error::Pool(e.to_string()))
    }

    /// Convert domain Patient model to database models
    fn to_db_models(&self, patient: &Patient) -> (NewDbPatient, Vec<NewDbPatientName>, Vec<NewDbPatientIdentifier>, Vec<NewDbPatientAddress>, Vec<NewDbPatientContact>, Vec<NewDbPatientLink>) {
        let new_patient = NewDbPatient {
            id: Some(patient.id),
            active: patient.active,
            gender: format!("{:?}", patient.gender),
            birth_date: patient.birth_date,
            deceased: patient.deceased,
            deceased_datetime: patient.deceased_datetime,
            marital_status: patient.marital_status.clone(),
            multiple_birth: patient.multiple_birth,
            managing_organization_id: patient.managing_organization,
            created_by: None, // TODO: Get from context
        };

        // Primary name
        let mut names = vec![NewDbPatientName {
            patient_id: patient.id,
            use_type: patient.name.use_type.as_ref().map(|u| format!("{:?}", u)),
            family: patient.name.family.clone(),
            given: patient.name.given.clone(),
            prefix: patient.name.prefix.clone(),
            suffix: patient.name.suffix.clone(),
            is_primary: true,
        }];

        // Additional names
        for add_name in &patient.additional_names {
            names.push(NewDbPatientName {
                patient_id: patient.id,
                use_type: add_name.use_type.as_ref().map(|u| format!("{:?}", u)),
                family: add_name.family.clone(),
                given: add_name.given.clone(),
                prefix: add_name.prefix.clone(),
                suffix: add_name.suffix.clone(),
                is_primary: false,
            });
        }

        // Identifiers
        let identifiers = patient.identifiers.iter().map(|id| NewDbPatientIdentifier {
            patient_id: patient.id,
            use_type: id.use_type.as_ref().map(|u| format!("{:?}", u)),
            identifier_type: format!("{:?}", id.identifier_type),
            system: id.system.clone(),
            value: id.value.clone(),
            assigner: id.assigner.clone(),
        }).collect();

        // Addresses
        let addresses = patient.addresses.iter().enumerate().map(|(idx, addr)| NewDbPatientAddress {
            patient_id: patient.id,
            use_type: None, // Not in domain model
            line1: addr.line1.clone(),
            line2: addr.line2.clone(),
            city: addr.city.clone(),
            state: addr.state.clone(),
            postal_code: addr.postal_code.clone(),
            country: addr.country.clone(),
            is_primary: idx == 0,
        }).collect();

        // Contacts
        let contacts = patient.telecom.iter().enumerate().map(|(idx, cp)| NewDbPatientContact {
            patient_id: patient.id,
            system: format!("{:?}", cp.system),
            value: cp.value.clone(),
            use_type: cp.use_type.as_ref().map(|u| format!("{:?}", u)),
            is_primary: idx == 0,
        }).collect();

        // Links
        let links = patient.links.iter().map(|link| NewDbPatientLink {
            patient_id: patient.id,
            other_patient_id: link.other_patient_id,
            link_type: format!("{:?}", link.link_type),
            created_by: None, // TODO: Get from context
        }).collect();

        (new_patient, names, identifiers, addresses, contacts, links)
    }

    /// Convert database models to domain Patient model
    fn from_db_models(
        &self,
        db_patient: DbPatient,
        db_names: Vec<DbPatientName>,
        db_identifiers: Vec<DbPatientIdentifier>,
        db_addresses: Vec<DbPatientAddress>,
        db_contacts: Vec<DbPatientContact>,
        db_links: Vec<DbPatientLink>,
    ) -> Result<Patient> {
        use crate::models::{Gender, NameUse, ContactPointSystem, ContactPointUse, LinkType, IdentifierType, IdentifierUse};

        // Parse gender
        let gender = match db_patient.gender.as_str() {
            "Male" => Gender::Male,
            "Female" => Gender::Female,
            "Other" => Gender::Other,
            _ => Gender::Unknown,
        };

        // Get primary name
        let primary_name = db_names.iter()
            .find(|n| n.is_primary)
            .ok_or_else(|| crate::Error::Validation("Patient has no primary name".to_string()))?;

        let name = HumanName {
            use_type: primary_name.use_type.as_ref().and_then(|u| match u.as_str() {
                "Usual" => Some(NameUse::Usual),
                "Official" => Some(NameUse::Official),
                "Temp" => Some(NameUse::Temp),
                "Nickname" => Some(NameUse::Nickname),
                "Anonymous" => Some(NameUse::Anonymous),
                "Old" => Some(NameUse::Old),
                "Maiden" => Some(NameUse::Maiden),
                _ => None,
            }),
            family: primary_name.family.clone(),
            given: primary_name.given.clone(),
            prefix: primary_name.prefix.clone(),
            suffix: primary_name.suffix.clone(),
        };

        // Additional names
        let additional_names = db_names.iter()
            .filter(|n| !n.is_primary)
            .map(|n| HumanName {
                use_type: n.use_type.as_ref().and_then(|u| match u.as_str() {
                    "Usual" => Some(NameUse::Usual),
                    "Official" => Some(NameUse::Official),
                    "Temp" => Some(NameUse::Temp),
                    "Nickname" => Some(NameUse::Nickname),
                    "Anonymous" => Some(NameUse::Anonymous),
                    "Old" => Some(NameUse::Old),
                    "Maiden" => Some(NameUse::Maiden),
                    _ => None,
                }),
                family: n.family.clone(),
                given: n.given.clone(),
                prefix: n.prefix.clone(),
                suffix: n.suffix.clone(),
            })
            .collect();

        // Identifiers
        let identifiers = db_identifiers.iter()
            .map(|id| {
                let identifier_type = match id.identifier_type.as_str() {
                    "MRN" => IdentifierType::MRN,
                    "SSN" => IdentifierType::SSN,
                    "DL" => IdentifierType::DL,
                    "NPI" => IdentifierType::NPI,
                    "PPN" => IdentifierType::PPN,
                    "TAX" => IdentifierType::TAX,
                    _ => IdentifierType::Other,
                };

                let use_type = id.use_type.as_ref().and_then(|u| match u.as_str() {
                    "Usual" => Some(IdentifierUse::Usual),
                    "Official" => Some(IdentifierUse::Official),
                    "Temp" => Some(IdentifierUse::Temp),
                    "Secondary" => Some(IdentifierUse::Secondary),
                    "Old" => Some(IdentifierUse::Old),
                    _ => None,
                });

                Identifier {
                    identifier_type,
                    use_type,
                    system: id.system.clone(),
                    value: id.value.clone(),
                    assigner: id.assigner.clone(),
                }
            })
            .collect();

        // Addresses
        let addresses = db_addresses.iter()
            .map(|addr| Address {
                line1: addr.line1.clone(),
                line2: addr.line2.clone(),
                city: addr.city.clone(),
                state: addr.state.clone(),
                postal_code: addr.postal_code.clone(),
                country: addr.country.clone(),
            })
            .collect();

        // Telecom
        let telecom = db_contacts.iter()
            .filter_map(|cp| {
                let system = match cp.system.as_str() {
                    "Phone" => ContactPointSystem::Phone,
                    "Fax" => ContactPointSystem::Fax,
                    "Email" => ContactPointSystem::Email,
                    "Pager" => ContactPointSystem::Pager,
                    "Url" => ContactPointSystem::Url,
                    "Sms" => ContactPointSystem::Sms,
                    "Other" => ContactPointSystem::Other,
                    _ => return None,
                };

                let use_type = cp.use_type.as_ref().and_then(|u| match u.as_str() {
                    "Home" => Some(ContactPointUse::Home),
                    "Work" => Some(ContactPointUse::Work),
                    "Temp" => Some(ContactPointUse::Temp),
                    "Old" => Some(ContactPointUse::Old),
                    "Mobile" => Some(ContactPointUse::Mobile),
                    _ => None,
                });

                Some(ContactPoint {
                    system,
                    value: cp.value.clone(),
                    use_type,
                })
            })
            .collect();

        // Links
        let links = db_links.iter()
            .filter_map(|link| {
                let link_type = match link.link_type.as_str() {
                    "ReplacedBy" => LinkType::ReplacedBy,
                    "Replaces" => LinkType::Replaces,
                    "Refer" => LinkType::Refer,
                    "Seealso" => LinkType::Seealso,
                    _ => return None,
                };

                Some(PatientLink {
                    other_patient_id: link.other_patient_id,
                    link_type,
                })
            })
            .collect();

        Ok(Patient {
            id: db_patient.id,
            identifiers,
            active: db_patient.active,
            name,
            additional_names,
            telecom,
            gender,
            birth_date: db_patient.birth_date,
            deceased: db_patient.deceased,
            deceased_datetime: db_patient.deceased_datetime,
            addresses,
            marital_status: db_patient.marital_status,
            multiple_birth: db_patient.multiple_birth,
            photo: vec![], // Not stored in DB yet
            managing_organization: db_patient.managing_organization_id,
            links,
            created_at: db_patient.created_at,
            updated_at: db_patient.updated_at,
        })
    }
}

impl PatientRepository for DieselPatientRepository {
    fn create(&self, patient: &Patient) -> Result<Patient> {
        let mut conn = self.get_conn()?;

        let result = conn.transaction(|conn| {
            let (new_patient, new_names, new_identifiers, new_addresses, new_contacts, new_links) =
                self.to_db_models(patient);

            // Insert patient
            let db_patient: DbPatient = diesel::insert_into(patients::table)
                .values(&new_patient)
                .get_result(conn)?;

            // Insert names
            let db_names: Vec<DbPatientName> = diesel::insert_into(patient_names::table)
                .values(&new_names)
                .get_results(conn)?;

            // Insert identifiers
            let db_identifiers: Vec<DbPatientIdentifier> = if !new_identifiers.is_empty() {
                diesel::insert_into(patient_identifiers::table)
                    .values(&new_identifiers)
                    .get_results(conn)?
            } else {
                vec![]
            };

            // Insert addresses
            let db_addresses: Vec<DbPatientAddress> = if !new_addresses.is_empty() {
                diesel::insert_into(patient_addresses::table)
                    .values(&new_addresses)
                    .get_results(conn)?
            } else {
                vec![]
            };

            // Insert contacts
            let db_contacts: Vec<DbPatientContact> = if !new_contacts.is_empty() {
                diesel::insert_into(patient_contacts::table)
                    .values(&new_contacts)
                    .get_results(conn)?
            } else {
                vec![]
            };

            // Insert links
            let db_links: Vec<DbPatientLink> = if !new_links.is_empty() {
                diesel::insert_into(patient_links::table)
                    .values(&new_links)
                    .get_results(conn)?
            } else {
                vec![]
            };

            self.from_db_models(db_patient, db_names, db_identifiers, db_addresses, db_contacts, db_links)
        })?;

        // Publish event
        self.publish_event(crate::streaming::PatientEvent::Created {
            patient: result.clone(),
            timestamp: chrono::Utc::now(),
        });

        // Log audit
        if let Ok(patient_json) = serde_json::to_value(&result) {
            self.log_audit("CREATE", result.id, None, Some(patient_json), &AuditContext::default());
        }

        Ok(result)
    }

    fn get_by_id(&self, id: &Uuid) -> Result<Option<Patient>> {
        let mut conn = self.get_conn()?;

        // Get patient
        let db_patient: Option<DbPatient> = patients::table
            .filter(patients::id.eq(id))
            .filter(patients::deleted_at.is_null())
            .first(&mut conn)
            .optional()?;

        let db_patient = match db_patient {
            Some(p) => p,
            None => return Ok(None),
        };

        // Get associated data
        let db_names: Vec<DbPatientName> = patient_names::table
            .filter(patient_names::patient_id.eq(id))
            .load(&mut conn)?;

        let db_identifiers: Vec<DbPatientIdentifier> = patient_identifiers::table
            .filter(patient_identifiers::patient_id.eq(id))
            .load(&mut conn)?;

        let db_addresses: Vec<DbPatientAddress> = patient_addresses::table
            .filter(patient_addresses::patient_id.eq(id))
            .load(&mut conn)?;

        let db_contacts: Vec<DbPatientContact> = patient_contacts::table
            .filter(patient_contacts::patient_id.eq(id))
            .load(&mut conn)?;

        let db_links: Vec<DbPatientLink> = patient_links::table
            .filter(patient_links::patient_id.eq(id))
            .load(&mut conn)?;

        self.from_db_models(db_patient, db_names, db_identifiers, db_addresses, db_contacts, db_links)
            .map(Some)
    }

    fn update(&self, patient: &Patient) -> Result<Patient> {
        let mut conn = self.get_conn()?;

        // Get old values for audit
        let old_patient = self.get_by_id(&patient.id)?;

        let result = conn.transaction(|conn| {
            // Update patient
            let update_patient = UpdateDbPatient {
                active: Some(patient.active),
                gender: Some(format!("{:?}", patient.gender)),
                birth_date: patient.birth_date,
                deceased: Some(patient.deceased),
                deceased_datetime: patient.deceased_datetime,
                marital_status: patient.marital_status.clone(),
                multiple_birth: patient.multiple_birth,
                managing_organization_id: patient.managing_organization,
                updated_by: None, // TODO: Get from context
            };

            diesel::update(patients::table.filter(patients::id.eq(patient.id)))
                .set(&update_patient)
                .execute(conn)?;

            // Delete existing associated data
            diesel::delete(patient_names::table.filter(patient_names::patient_id.eq(patient.id)))
                .execute(conn)?;

            diesel::delete(patient_identifiers::table.filter(patient_identifiers::patient_id.eq(patient.id)))
                .execute(conn)?;

            diesel::delete(patient_addresses::table.filter(patient_addresses::patient_id.eq(patient.id)))
                .execute(conn)?;

            diesel::delete(patient_contacts::table.filter(patient_contacts::patient_id.eq(patient.id)))
                .execute(conn)?;

            diesel::delete(patient_links::table.filter(patient_links::patient_id.eq(patient.id)))
                .execute(conn)?;

            // Re-insert associated data
            let (_, new_names, new_identifiers, new_addresses, new_contacts, new_links) =
                self.to_db_models(patient);

            diesel::insert_into(patient_names::table)
                .values(&new_names)
                .execute(conn)?;

            if !new_identifiers.is_empty() {
                diesel::insert_into(patient_identifiers::table)
                    .values(&new_identifiers)
                    .execute(conn)?;
            }

            if !new_addresses.is_empty() {
                diesel::insert_into(patient_addresses::table)
                    .values(&new_addresses)
                    .execute(conn)?;
            }

            if !new_contacts.is_empty() {
                diesel::insert_into(patient_contacts::table)
                    .values(&new_contacts)
                    .execute(conn)?;
            }

            if !new_links.is_empty() {
                diesel::insert_into(patient_links::table)
                    .values(&new_links)
                    .execute(conn)?;
            }

            // Fetch and return updated patient
            self.get_by_id(&patient.id)?
                .ok_or_else(|| crate::Error::Validation("Patient not found after update".to_string()))
        })?;

        // Publish event
        self.publish_event(crate::streaming::PatientEvent::Updated {
            patient: result.clone(),
            timestamp: chrono::Utc::now(),
        });

        // Log audit
        if let Some(old_json) = old_patient.as_ref().and_then(|p| serde_json::to_value(p).ok()) {
            if let Ok(new_json) = serde_json::to_value(&result) {
                self.log_audit("UPDATE", result.id, Some(old_json), Some(new_json), &AuditContext::default());
            }
        }

        Ok(result)
    }

    fn delete(&self, id: &Uuid) -> Result<()> {
        let mut conn = self.get_conn()?;

        // Get old values for audit
        let old_patient = self.get_by_id(id)?;

        // Soft delete
        diesel::update(patients::table.filter(patients::id.eq(id)))
            .set((
                patients::deleted_at.eq(Some(Utc::now())),
                patients::deleted_by.eq(Some("system".to_string())), // TODO: Get from context
            ))
            .execute(&mut conn)?;

        // Publish event
        self.publish_event(crate::streaming::PatientEvent::Deleted {
            patient_id: *id,
            timestamp: chrono::Utc::now(),
        });

        // Log audit
        if let Some(old_patient) = old_patient {
            if let Ok(old_json) = serde_json::to_value(&old_patient) {
                self.log_audit("DELETE", *id, Some(old_json), None, &AuditContext::default());
            }
        }

        Ok(())
    }

    fn search(&self, query: &str) -> Result<Vec<Patient>> {
        let mut conn = self.get_conn()?;

        // Search by family name (simple implementation)
        let search_pattern = format!("%{}%", query.to_lowercase());

        let patient_ids: Vec<Uuid> = patient_names::table
            .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!("LOWER(family) LIKE '{}'", search_pattern)))
            .select(patient_names::patient_id)
            .distinct()
            .load(&mut conn)?;

        // Fetch full patient records
        let mut patients = Vec::new();
        for patient_id in patient_ids {
            if let Some(patient) = self.get_by_id(&patient_id)? {
                patients.push(patient);
            }
        }

        Ok(patients)
    }

    fn list_active(&self, limit: i64, offset: i64) -> Result<Vec<Patient>> {
        let mut conn = self.get_conn()?;

        let patient_ids: Vec<Uuid> = patients::table
            .filter(patients::deleted_at.is_null())
            .filter(patients::active.eq(true))
            .select(patients::id)
            .limit(limit)
            .offset(offset)
            .load(&mut conn)?;

        let mut patients = Vec::new();
        for patient_id in patient_ids {
            if let Some(patient) = self.get_by_id(&patient_id)? {
                patients.push(patient);
            }
        }

        Ok(patients)
    }
}
