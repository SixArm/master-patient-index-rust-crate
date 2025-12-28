//! RESTful API implementation with Axum

use axum::{
    Router,
    routing::{get, post, put, delete},
};
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod handlers;
pub mod routes;

use crate::config::ServerConfig;
use crate::Result;

/// API documentation
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Master Patient Index API",
        version = "0.1.0",
        description = "RESTful API for patient identification and matching",
        contact(
            name = "MPI Development Team",
            email = "support@example.com"
        )
    ),
    paths(
        // TODO: Add API endpoint paths
    ),
    components(
        schemas(
            crate::models::Patient,
            crate::models::patient::HumanName,
            crate::models::Organization,
            crate::models::Identifier,
            crate::api::ApiResponse::<crate::models::Patient>,
            crate::api::ApiError,
        )
    ),
    tags(
        (name = "patients", description = "Patient management endpoints"),
        (name = "search", description = "Patient search endpoints"),
        (name = "matching", description = "Patient matching endpoints"),
        (name = "organizations", description = "Organization management endpoints")
    )
)]
pub struct ApiDoc;

/// Create the REST API router
pub fn create_router() -> Router {
    let api_routes = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/patients", post(handlers::create_patient))
        .route("/patients/:id", get(handlers::get_patient))
        .route("/patients/:id", put(handlers::update_patient))
        .route("/patients/:id", delete(handlers::delete_patient))
        .route("/patients/search", get(handlers::search_patients))
        .route("/patients/match", post(handlers::match_patient));

    Router::new()
        .nest("/api/v1", api_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(CorsLayer::permissive())
}

/// Start the REST API server
pub async fn serve(config: ServerConfig) -> Result<()> {
    let app = create_router();
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| crate::Error::Api(e.to_string()))?;

    tracing::info!("REST API server listening on {}", addr);

    axum::serve(listener, app)
        .await
        .map_err(|e| crate::Error::Api(e.to_string()))?;

    Ok(())
}
