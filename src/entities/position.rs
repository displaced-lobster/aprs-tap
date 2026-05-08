use sea_orm::Set;
use sea_orm::entity::prelude::*;
use serde::Serialize;

use super::user;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "positions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub user_id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude_m: Option<i32>,
    pub symbol_table: String,
    pub symbol_code: String,
    pub comment: Option<String>,
    pub date_created: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "user::Entity",
        from = "Column::UserId",
        to = "user::Column::Id",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if insert {
            self.date_created = Set(chrono::Utc::now().naive_utc());
        }
        Ok(self)
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct PositionResponse {
    pub id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude_m: Option<i32>,
    pub symbol_table: String,
    pub symbol_code: String,
    pub comment: Option<String>,
    pub date_created: chrono::NaiveDateTime,
}

impl From<Model> for PositionResponse {
    fn from(m: Model) -> Self {
        Self {
            id: m.id,
            latitude: m.latitude,
            longitude: m.longitude,
            altitude_m: m.altitude_m,
            symbol_table: m.symbol_table,
            symbol_code: m.symbol_code,
            comment: m.comment,
            date_created: m.date_created,
        }
    }
}
