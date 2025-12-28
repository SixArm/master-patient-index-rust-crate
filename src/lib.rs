//! Master Patient Index (MPI)
//!
//! A healthcare patient identification and matching system built with Rust.
//!
//! This library provides:
//! - Patient matching algorithms (probabilistic and deterministic)
//! - Full-text search capabilities via Tantivy
//! - RESTful API via Axum
//! - HL7 FHIR R5 support
//! - gRPC API via Tonic
//! - PostgreSQL persistence via Diesel
//! - Event streaming via Fluvio
//! - Distributed tracing and observability via OpenTelemetry

// Module declarations
pub mod api;
pub mod config;
pub mod db;
pub mod error;
pub mod matching;
pub mod models;
pub mod observability;
pub mod search;
pub mod streaming;

// Re-exports
pub use error::{Error, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_imports() {
        // Verify all modules are accessible
    }
}
