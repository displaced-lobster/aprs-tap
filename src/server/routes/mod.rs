pub mod auth;
pub mod positions;

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde::Serialize;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::server::{db::AppState, worker::WorkerStatus};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/auth/signup", post(auth::signup))
        .route("/auth/login", post(auth::login))
        .route("/positions", get(positions::list))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health,
        auth::signup,
        auth::login,
        positions::list,
    ),
    components(schemas(
        HealthResponse,
        WorkerStatus,
        auth::SignupRequest,
        auth::LoginRequest,
        auth::AuthResponse,
        crate::server::entities::user::UserResponse,
        crate::server::entities::position::PositionResponse,
    )),
    tags(
        (name = "auth", description = "Registration and login"),
        (name = "health", description = "Health check"),
        (name = "positions", description = "Position history"),
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
async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let worker = state.worker_status.read().unwrap().clone();
    Json(HealthResponse {
        status: "ok",
        worker,
    })
}

#[derive(Serialize, utoipa::ToSchema)]
struct HealthResponse {
    status: &'static str,
    worker: WorkerStatus,
}
