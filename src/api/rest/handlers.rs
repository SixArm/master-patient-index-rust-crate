//! REST API request handlers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;
use chrono::Datelike;

use crate::models::Patient;
use crate::api::{ApiResponse, ApiError};
use crate::matching::MatchResult;
use super::state::AppState;

/// Health check response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
pub async fn health_check() -> impl IntoResponse {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "master-patient-index".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Create patient request
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePatientRequest {
    #[serde(flatten)]
    pub patient: Patient,
}

/// Create a new patient
#[utoipa::path(
    post,
    path = "/api/v1/patients",
    tag = "patients",
    request_body = Patient,
    responses(
        (status = 201, description = "Patient created successfully"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_patient(
    State(state): State<AppState>,
    Json(mut payload): Json<Patient>,
) -> impl IntoResponse {
    // Ensure patient has a UUID
    if payload.id == Uuid::nil() {
        payload.id = Uuid::new_v4();
    }

    // Insert into database
    match state.patient_repository.create(&payload) {
        Ok(patient) => {
            // Index in search engine
            if let Err(e) = state.search_engine.index_patient(&patient) {
                tracing::warn!("Failed to index patient in search engine: {}", e);
            }

            (StatusCode::CREATED, Json(ApiResponse::success(patient)))
        }
        Err(e) => {
            let error = ApiResponse::<Patient>::error(
                "DATABASE_ERROR",
                format!("Failed to create patient: {}", e)
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

/// Get a patient by ID
#[utoipa::path(
    get,
    path = "/api/v1/patients/{id}",
    tag = "patients",
    params(
        ("id" = Uuid, Path, description = "Patient UUID")
    ),
    responses(
        (status = 200, description = "Patient found"),
        (status = 404, description = "Patient not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_patient(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.patient_repository.get_by_id(&id) {
        Ok(Some(patient)) => {
            (StatusCode::OK, Json(ApiResponse::success(patient)))
        }
        Ok(None) => {
            let error = ApiResponse::<Patient>::error(
                "NOT_FOUND",
                format!("Patient with id '{}' not found", id)
            );
            (StatusCode::NOT_FOUND, Json(error))
        }
        Err(e) => {
            let error = ApiResponse::<Patient>::error(
                "DATABASE_ERROR",
                format!("Failed to retrieve patient: {}", e)
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

/// Update a patient
#[utoipa::path(
    put,
    path = "/api/v1/patients/{id}",
    tag = "patients",
    params(
        ("id" = Uuid, Path, description = "Patient UUID")
    ),
    request_body = Patient,
    responses(
        (status = 200, description = "Patient updated successfully"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_patient(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(mut payload): Json<Patient>,
) -> impl IntoResponse {
    // Ensure ID in path matches payload
    payload.id = id;

    match state.patient_repository.update(&payload) {
        Ok(patient) => {
            // Update search index
            if let Err(e) = state.search_engine.index_patient(&patient) {
                tracing::warn!("Failed to update patient in search engine: {}", e);
            }

            (StatusCode::OK, Json(ApiResponse::success(patient)))
        }
        Err(e) => {
            let error = ApiResponse::<Patient>::error(
                "DATABASE_ERROR",
                format!("Failed to update patient: {}", e)
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

/// Delete a patient (soft delete)
#[utoipa::path(
    delete,
    path = "/api/v1/patients/{id}",
    tag = "patients",
    params(
        ("id" = Uuid, Path, description = "Patient UUID")
    ),
    responses(
        (status = 204, description = "Patient deleted successfully"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_patient(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.patient_repository.delete(&id) {
        Ok(()) => {
            // Remove from search index
            if let Err(e) = state.search_engine.delete_patient(&id.to_string()) {
                tracing::warn!("Failed to delete patient from search engine: {}", e);
            }

            (StatusCode::NO_CONTENT, Json(ApiResponse::<()>::success(())))
        }
        Err(e) => {
            let error = ApiResponse::<()>::error(
                "DATABASE_ERROR",
                format!("Failed to delete patient: {}", e)
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

/// Search query parameters
#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct SearchQuery {
    /// Search query string
    pub q: String,

    /// Maximum number of results (default: 10, max: 100)
    #[serde(default = "default_limit")]
    pub limit: usize,

    /// Use fuzzy search
    #[serde(default)]
    pub fuzzy: bool,
}

fn default_limit() -> usize {
    10
}

/// Search results response
#[derive(Debug, Serialize, ToSchema)]
pub struct SearchResponse {
    pub patients: Vec<Patient>,
    pub total: usize,
    pub query: String,
}

/// Search for patients
#[utoipa::path(
    get,
    path = "/api/v1/patients/search",
    tag = "search",
    params(SearchQuery),
    responses(
        (status = 200, description = "Search results", body = SearchResponse),
        (status = 500, description = "Search error")
    )
)]
pub async fn search_patients(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    // Limit to max 100 results
    let limit = params.limit.min(100);

    // Perform search using search engine
    let patient_ids = if params.fuzzy {
        state.search_engine.fuzzy_search(&params.q, limit)
    } else {
        state.search_engine.search(&params.q, limit)
    };

    match patient_ids {
        Ok(ids) => {
            // Fetch full patient records from database
            let mut patients = Vec::new();
            for patient_id_str in ids {
                // Parse string ID to UUID
                let patient_id = match Uuid::parse_str(&patient_id_str) {
                    Ok(id) => id,
                    Err(e) => {
                        tracing::error!("Failed to parse patient ID {}: {}", patient_id_str, e);
                        continue;
                    }
                };

                match state.patient_repository.get_by_id(&patient_id) {
                    Ok(Some(patient)) => patients.push(patient),
                    Ok(None) => {
                        tracing::warn!("Patient {} found in search index but not in database", patient_id);
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch patient {}: {}", patient_id, e);
                    }
                }
            }

            let response = SearchResponse {
                total: patients.len(),
                patients,
                query: params.q,
            };
            (StatusCode::OK, Json(ApiResponse::success(response)))
        }
        Err(e) => {
            let error = ApiResponse::<SearchResponse>::error(
                "SEARCH_ERROR",
                format!("Search failed: {}", e)
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

/// Match request payload
#[derive(Debug, Deserialize, ToSchema)]
pub struct MatchRequest {
    /// Patient to match against existing records
    #[serde(flatten)]
    pub patient: Patient,

    /// Minimum match score threshold (0.0 to 1.0)
    #[serde(default)]
    pub threshold: Option<f64>,

    /// Maximum number of matches to return
    #[serde(default = "default_match_limit")]
    pub limit: usize,
}

fn default_match_limit() -> usize {
    10
}

/// Match result with score
#[derive(Debug, Serialize, ToSchema)]
pub struct MatchResponse {
    pub patient: Patient,
    pub score: f64,
    pub quality: String,
}

/// Match results response
#[derive(Debug, Serialize, ToSchema)]
pub struct MatchResultsResponse {
    pub matches: Vec<MatchResponse>,
    pub total: usize,
}

/// Match a patient against existing records
#[utoipa::path(
    post,
    path = "/api/v1/patients/match",
    tag = "matching",
    request_body = MatchRequest,
    responses(
        (status = 200, description = "Match results", body = MatchResultsResponse),
        (status = 500, description = "Matching error")
    )
)]
pub async fn match_patient(
    State(state): State<AppState>,
    Json(payload): Json<MatchRequest>,
) -> impl IntoResponse {
    // Use search engine to get candidate patients (blocking)
    let family_name = &payload.patient.name.family;
    let birth_year = payload.patient.birth_date.map(|d| d.year());

    let candidate_ids = state.search_engine
        .search_by_name_and_year(family_name, birth_year, 100);

    match candidate_ids {
        Ok(ids) => {
            // Fetch full patient records from database
            let mut candidates = Vec::new();
            for patient_id_str in ids {
                // Parse string ID to UUID
                let patient_id = match Uuid::parse_str(&patient_id_str) {
                    Ok(id) => id,
                    Err(e) => {
                        tracing::error!("Failed to parse patient ID {}: {}", patient_id_str, e);
                        continue;
                    }
                };

                match state.patient_repository.get_by_id(&patient_id) {
                    Ok(Some(patient)) => candidates.push(patient),
                    Ok(None) => {
                        tracing::warn!("Patient {} found in search index but not in database", patient_id);
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch patient {}: {}", patient_id, e);
                    }
                }
            }

            // Run matcher on candidates
            let match_results = match state.matcher.find_matches(&payload.patient, &candidates) {
                Ok(results) => results,
                Err(e) => {
                    let error = ApiResponse::<MatchResultsResponse>::error(
                        "MATCH_ERROR",
                        format!("Matching failed: {}", e)
                    );
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(error));
                }
            };

            // Filter by threshold if provided
            let threshold = payload.threshold.unwrap_or(0.5);
            let matches: Vec<MatchResponse> = match_results.into_iter()
                .filter(|m| m.score >= threshold)
                .take(payload.limit)
                .map(|m| {
                    let quality = if m.score >= 0.9 {
                        "certain"
                    } else if m.score >= 0.7 {
                        "probable"
                    } else {
                        "possible"
                    };

                    MatchResponse {
                        patient: m.patient.clone(),
                        score: m.score,
                        quality: quality.to_string(),
                    }
                })
                .collect();

            let response = MatchResultsResponse {
                total: matches.len(),
                matches,
            };
            (StatusCode::OK, Json(ApiResponse::success(response)))
        }
        Err(e) => {
            let error = ApiResponse::<MatchResultsResponse>::error(
                "MATCH_ERROR",
                format!("Matching failed: {}", e)
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

/// Audit log query parameters
#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct AuditLogQuery {
    /// Maximum number of results (default: 50, max: 500)
    #[serde(default = "default_audit_limit")]
    pub limit: i64,
}

fn default_audit_limit() -> i64 {
    50
}

/// Get audit logs for a specific patient
#[utoipa::path(
    get,
    path = "/api/v1/patients/{id}/audit",
    tag = "audit",
    params(
        ("id" = Uuid, Path, description = "Patient UUID"),
        AuditLogQuery
    ),
    responses(
        (status = 200, description = "Audit logs retrieved successfully"),
        (status = 500, description = "Database error")
    )
)]
pub async fn get_patient_audit_logs(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<AuditLogQuery>,
) -> impl IntoResponse {
    let limit = params.limit.min(500);

    match state.audit_log.get_logs_for_entity("patient", id, limit) {
        Ok(logs) => (StatusCode::OK, Json(ApiResponse::success(logs))),
        Err(e) => {
            let error = ApiResponse::<Vec<crate::db::models::DbAuditLog>>::error(
                "DATABASE_ERROR",
                format!("Failed to retrieve audit logs: {}", e)
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

/// Get recent audit logs
#[utoipa::path(
    get,
    path = "/api/v1/audit/recent",
    tag = "audit",
    params(AuditLogQuery),
    responses(
        (status = 200, description = "Recent audit logs retrieved successfully"),
        (status = 500, description = "Database error")
    )
)]
pub async fn get_recent_audit_logs(
    State(state): State<AppState>,
    Query(params): Query<AuditLogQuery>,
) -> impl IntoResponse {
    let limit = params.limit.min(500);

    match state.audit_log.get_recent_logs(limit) {
        Ok(logs) => (StatusCode::OK, Json(ApiResponse::success(logs))),
        Err(e) => {
            let error = ApiResponse::<Vec<crate::db::models::DbAuditLog>>::error(
                "DATABASE_ERROR",
                format!("Failed to retrieve audit logs: {}", e)
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}

/// User audit log query parameters
#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct UserAuditLogQuery {
    /// User ID to filter by
    pub user_id: String,

    /// Maximum number of results (default: 50, max: 500)
    #[serde(default = "default_audit_limit")]
    pub limit: i64,
}

/// Get audit logs by user
#[utoipa::path(
    get,
    path = "/api/v1/audit/user",
    tag = "audit",
    params(UserAuditLogQuery),
    responses(
        (status = 200, description = "User audit logs retrieved successfully"),
        (status = 500, description = "Database error")
    )
)]
pub async fn get_user_audit_logs(
    State(state): State<AppState>,
    Query(params): Query<UserAuditLogQuery>,
) -> impl IntoResponse {
    let limit = params.limit.min(500);

    match state.audit_log.get_logs_by_user(&params.user_id, limit) {
        Ok(logs) => (StatusCode::OK, Json(ApiResponse::success(logs))),
        Err(e) => {
            let error = ApiResponse::<Vec<crate::db::models::DbAuditLog>>::error(
                "DATABASE_ERROR",
                format!("Failed to retrieve audit logs: {}", e)
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
        }
    }
}
