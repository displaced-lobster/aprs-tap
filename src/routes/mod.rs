pub mod auth;

use axum::{Json, Router, routing::{get, post}};
use serde::Serialize;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::db::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/auth/signup", post(auth::signup))
        .route("/auth/login", post(auth::login))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health,
        auth::signup,
        auth::login,
    ),
    components(schemas(
        HealthResponse,
        auth::SignupRequest,
        auth::LoginRequest,
        auth::AuthResponse,
        crate::entities::user::UserResponse,
    )),
    tags(
        (name = "auth", description = "Registration and login"),
        (name = "health", description = "Health check"),
    ),
    info(title = "aprs-tap", version = "0.1.0"),
)]
struct ApiDoc;

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Server is healthy", body = HealthResponse),
    ),
    tag = "health",
)]
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

#[derive(Serialize, utoipa::ToSchema)]
struct HealthResponse {
    status: &'static str,
}
