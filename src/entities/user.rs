use sea_orm::Set;
use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(unique)]
    pub callsign: String,
    pub password_hash: String,
    #[sea_orm(unique)]
    pub slug: String,
    pub date_created: chrono::NaiveDateTime,
    pub date_modified: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        let now = chrono::Utc::now().naive_utc();
        self.date_modified = Set(now);
        if insert {
            self.date_created = Set(now);
        }
        Ok(self)
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct UserResponse {
    pub callsign: String,
    pub slug: String,
}

impl From<Model> for UserResponse {
    fn from(m: Model) -> Self {
        Self {
            callsign: m.callsign,
            slug: m.slug,
        }
    }
}
