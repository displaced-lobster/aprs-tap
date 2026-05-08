use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use rand::Rng;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::{create_token, hash_password, verify_password},
    db::AppState,
    entities::user,
    error::AppError,
};

#[derive(Deserialize)]
pub struct SignupRequest {
    callsign: String,
    password: String,
    password_confirm: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    callsign: String,
    password: String,
}

#[derive(serde::Serialize)]
pub struct AuthResponse {
    user: user::UserResponse,
    token: String,
}

pub async fn signup(
    State(state): State<AppState>,
    Json(body): Json<SignupRequest>,
) -> Result<impl IntoResponse, AppError> {
    if body.password != body.password_confirm {
        return Err(AppError::BadRequest("passwords do not match"));
    }

    let callsign = body.callsign.to_uppercase();
    let password_hash = hash_password(&body.password).map_err(|e| {
        eprintln!("password hashing failed: {e}");
        AppError::Internal
    })?;

    let model = user::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        callsign: Set(callsign),
        password_hash: Set(password_hash),
        slug: Set(generate_slug()),
        ..Default::default()
    }
    .insert(&state.db)
    .await?;

    let token = create_token(&model.id, &model.callsign, &state.jwt_secret).map_err(|e| {
        eprintln!("token creation failed: {e}");
        AppError::Internal
    })?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            user: model.into(),
            token,
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let callsign = body.callsign.to_uppercase();

    let model = user::Entity::find()
        .filter(user::Column::Callsign.eq(&callsign))
        .one(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let verified = verify_password(&body.password, &model.password_hash).map_err(|e| {
        eprintln!("password verification failed: {e}");
        AppError::Internal
    })?;

    if !verified {
        return Err(AppError::Unauthorized);
    }

    let token = create_token(&model.id, &model.callsign, &state.jwt_secret).map_err(|e| {
        eprintln!("token creation failed: {e}");
        AppError::Internal
    })?;

    Ok(Json(AuthResponse {
        user: model.into(),
        token,
    }))
}

fn generate_slug() -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(8)
        .map(|b| (b as char).to_ascii_lowercase())
        .collect()
}
