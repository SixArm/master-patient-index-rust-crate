//! Audit log repository for tracking changes

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use uuid::Uuid;
use serde_json::Value as JsonValue;

use crate::Result;
use super::models::{NewDbAuditLog, DbAuditLog};
use super::schema::audit_log;

/// Audit log repository for recording changes
pub struct AuditLogRepository {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl AuditLogRepository {
    /// Create a new audit log repository
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }

    /// Get a database connection from the pool
    fn get_conn(&self) -> Result<diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>>> {
        self.pool.get().map_err(|e| crate::Error::Pool(e.to_string()))
    }

    /// Log a create action
    pub fn log_create(
        &self,
        entity_type: &str,
        entity_id: Uuid,
        new_values: JsonValue,
        user_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<()> {
        self.log_action(
            "CREATE",
            entity_type,
            entity_id,
            None,
            Some(new_values),
            user_id,
            ip_address,
            user_agent,
        )
    }

    /// Log an update action
    pub fn log_update(
        &self,
        entity_type: &str,
        entity_id: Uuid,
        old_values: JsonValue,
        new_values: JsonValue,
        user_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<()> {
        self.log_action(
            "UPDATE",
            entity_type,
            entity_id,
            Some(old_values),
            Some(new_values),
            user_id,
            ip_address,
            user_agent,
        )
    }

    /// Log a delete action
    pub fn log_delete(
        &self,
        entity_type: &str,
        entity_id: Uuid,
        old_values: JsonValue,
        user_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<()> {
        self.log_action(
            "DELETE",
            entity_type,
            entity_id,
            Some(old_values),
            None,
            user_id,
            ip_address,
            user_agent,
        )
    }

    /// Log a generic action
    fn log_action(
        &self,
        action: &str,
        entity_type: &str,
        entity_id: Uuid,
        old_values: Option<JsonValue>,
        new_values: Option<JsonValue>,
        user_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<()> {
        let mut conn = self.get_conn()?;

        let new_audit = NewDbAuditLog {
            user_id,
            action: action.to_string(),
            entity_type: entity_type.to_string(),
            entity_id,
            old_values,
            new_values,
            ip_address,
            user_agent,
        };

        diesel::insert_into(audit_log::table)
            .values(&new_audit)
            .execute(&mut conn)?;

        Ok(())
    }

    /// Get audit logs for a specific entity
    pub fn get_logs_for_entity(
        &self,
        entity_type: &str,
        entity_id: Uuid,
        limit: i64,
    ) -> Result<Vec<DbAuditLog>> {
        let mut conn = self.get_conn()?;

        let logs = audit_log::table
            .filter(audit_log::entity_type.eq(entity_type))
            .filter(audit_log::entity_id.eq(entity_id))
            .order(audit_log::timestamp.desc())
            .limit(limit)
            .load::<DbAuditLog>(&mut conn)?;

        Ok(logs)
    }

    /// Get recent audit logs
    pub fn get_recent_logs(&self, limit: i64) -> Result<Vec<DbAuditLog>> {
        let mut conn = self.get_conn()?;

        let logs = audit_log::table
            .order(audit_log::timestamp.desc())
            .limit(limit)
            .load::<DbAuditLog>(&mut conn)?;

        Ok(logs)
    }

    /// Get audit logs by user
    pub fn get_logs_by_user(
        &self,
        user_id: &str,
        limit: i64,
    ) -> Result<Vec<DbAuditLog>> {
        let mut conn = self.get_conn()?;

        let logs = audit_log::table
            .filter(audit_log::user_id.eq(user_id))
            .order(audit_log::timestamp.desc())
            .limit(limit)
            .load::<DbAuditLog>(&mut conn)?;

        Ok(logs)
    }
}
