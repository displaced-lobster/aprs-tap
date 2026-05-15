use axum::{
    Json,
    extract::{Query, State},
};
use sea_orm::{
    ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, sea_query::Condition,
};
use serde::Deserialize;

use crate::server::{
    db::AppState,
    entities::{
        position::{self, PositionResponse},
        user,
    },
    error::AppError,
};

#[derive(Deserialize, utoipa::IntoParams)]
pub struct ListParams {
    /// Filter by callsign (case-insensitive)
    pub callsign: Option<String>,
    #[param(default = 100, minimum = 1, maximum = 1000)]
    pub limit: Option<u64>,
    #[param(default = 0, minimum = 0)]
    pub offset: Option<u64>,
}

#[utoipa::path(
    get,
    path = "/positions",
    params(ListParams),
    responses(
        (status = 200, description = "Paginated list of positions", body = Vec<PositionResponse>),
    ),
    tag = "positions",
)]
pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<PositionResponse>>, AppError> {
    let limit = params.limit.unwrap_or(100).min(1000);
    let offset = params.offset.unwrap_or(0);

    let mut condition = Condition::all();
    if let Some(callsign) = params.callsign {
        let user = user::Entity::find()
            .filter(user::Column::Callsign.eq(callsign.to_uppercase()))
            .one(&state.db)
            .await?
            .ok_or(AppError::NotFound)?;
        condition = condition.add(position::Column::UserId.eq(user.id));
    }

    let positions = position::Entity::find()
        .filter(condition)
        .order_by_desc(position::Column::DateCreated)
        .limit(limit)
        .offset(offset)
        .all(&state.db)
        .await?
        .into_iter()
        .map(PositionResponse::from)
        .collect();

    Ok(Json(positions))
}
