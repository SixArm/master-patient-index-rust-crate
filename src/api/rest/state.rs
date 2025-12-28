//! Application state for REST API

use std::sync::Arc;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;

use crate::search::SearchEngine;
use crate::matching::ProbabilisticMatcher;
use crate::config::{Config, MatchingConfig};
use crate::Result;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool
    pub db_pool: Pool<ConnectionManager<PgConnection>>,

    /// Search engine for patient lookups
    pub search_engine: Arc<SearchEngine>,

    /// Patient matcher for finding duplicates
    pub matcher: Arc<ProbabilisticMatcher>,

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
        Self {
            db_pool,
            search_engine: Arc::new(search_engine),
            matcher: Arc::new(matcher),
            config: Arc::new(config),
        }
    }
}
