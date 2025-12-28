//! Application state for REST API

use std::sync::Arc;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;

use crate::search::SearchEngine;
use crate::matching::{ProbabilisticMatcher, PatientMatcher};
use crate::config::Config;
use crate::db::{PatientRepository, DieselPatientRepository};

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool
    pub db_pool: Pool<ConnectionManager<PgConnection>>,

    /// Patient repository for database operations
    pub patient_repository: Arc<dyn PatientRepository>,

    /// Search engine for patient lookups
    pub search_engine: Arc<SearchEngine>,

    /// Patient matcher for finding duplicates
    pub matcher: Arc<dyn PatientMatcher>,

    /// Application configuration
    pub config: Arc<Config>,
}

impl AppState {
    /// Create a new application state
    pub fn new(
        db_pool: Pool<ConnectionManager<PgConnection>>,
        search_engine: SearchEngine,
        matcher: ProbabilisticMatcher,
        config: Config,
    ) -> Self {
        let patient_repository = Arc::new(DieselPatientRepository::new(db_pool.clone())) as Arc<dyn PatientRepository>;
        let patient_matcher = Arc::new(matcher) as Arc<dyn PatientMatcher>;

        Self {
            db_pool,
            patient_repository,
            search_engine: Arc::new(search_engine),
            matcher: patient_matcher,
            config: Arc::new(config),
        }
    }
}
