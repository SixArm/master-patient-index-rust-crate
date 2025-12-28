//! REST API request handlers

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::Patient;
use crate::api::ApiResponse;

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "master-patient-index",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Create a new patient
pub async fn create_patient(
    Json(payload): Json<Patient>,
) -> impl IntoResponse {
    // TODO: Implement patient creation
    (StatusCode::CREATED, Json(ApiResponse::success(payload)))
}

/// Get a patient by ID
pub async fn get_patient(
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // TODO: Implement patient retrieval
    (StatusCode::NOT_IMPLEMENTED, Json(ApiResponse::<()>::error(
        "NOT_IMPLEMENTED",
        "Patient retrieval not yet implemented"
    )))
}

/// Update a patient
pub async fn update_patient(
    Path(id): Path<Uuid>,
    Json(payload): Json<Patient>,
) -> impl IntoResponse {
    // TODO: Implement patient update
    (StatusCode::NOT_IMPLEMENTED, Json(ApiResponse::<()>::error(
        "NOT_IMPLEMENTED",
        "Patient update not yet implemented"
    )))
}

/// Delete a patient (soft delete)
pub async fn delete_patient(
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // TODO: Implement patient deletion
    (StatusCode::NOT_IMPLEMENTED, Json(ApiResponse::<()>::error(
        "NOT_IMPLEMENTED",
        "Patient deletion not yet implemented"
    )))
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<usize>,
}

/// Search for patients
pub async fn search_patients(
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    // TODO: Implement patient search
    (StatusCode::NOT_IMPLEMENTED, Json(ApiResponse::<Vec<Patient>>::error(
        "NOT_IMPLEMENTED",
        "Patient search not yet implemented"
    )))
}

/// Match a patient against existing records
pub async fn match_patient(
    Json(payload): Json<Patient>,
) -> impl IntoResponse {
    // TODO: Implement patient matching
    (StatusCode::NOT_IMPLEMENTED, Json(ApiResponse::<Vec<Patient>>::error(
        "NOT_IMPLEMENTED",
        "Patient matching not yet implemented"
    )))
}
