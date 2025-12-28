//! Database operations and connection management

use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager, Pool};

use crate::config::DatabaseConfig;
use crate::Result;

pub mod schema;
pub mod models;
pub mod repositories;

pub use repositories::{PatientRepository, DieselPatientRepository};

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

/// Create a database connection pool
pub fn create_pool(config: &DatabaseConfig) -> Result<DbPool> {
    let manager = ConnectionManager::<PgConnection>::new(&config.url);

    Pool::builder()
        .max_size(config.max_connections)
        .min_idle(Some(config.min_connections))
        .build(manager)
        .map_err(|e| crate::Error::Pool(e.to_string()))
}

/// Get a database connection from the pool
pub fn get_connection(pool: &DbPool) -> Result<r2d2::PooledConnection<ConnectionManager<PgConnection>>> {
    pool.get()
        .map_err(|e| crate::Error::Pool(e.to_string()))
}
