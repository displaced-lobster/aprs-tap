use sea_orm::{ConnectOptions, DatabaseConnection};
use std::{sync::Arc, time::Duration};
use tokio::sync::Notify;

use crate::{migrator::Migrator, worker::StatusHandle};
use sea_orm_migration::MigratorTrait;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub jwt_secret: String,
    pub worker_notify: Arc<Notify>,
    pub worker_status: StatusHandle,
}

pub async fn connect(database_url: &str) -> Result<DatabaseConnection, sea_orm::DbErr> {
    // SQLite won't create the file unless told to; append mode=rwc if not already set.
    let url = if database_url.starts_with("sqlite") && !database_url.contains("mode=") {
        let sep = if database_url.contains('?') { '&' } else { '?' };
        format!("{database_url}{sep}mode=rwc")
    } else {
        database_url.to_owned()
    };

    let mut opt = ConnectOptions::new(url);

    // SQLite is single-writer; Postgres can handle a real pool.
    if database_url.starts_with("sqlite") {
        opt.max_connections(1).min_connections(1);
    } else {
        opt.max_connections(10).min_connections(2);
    }

    opt.connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(300))
        .sqlx_logging(false);

    let db = sea_orm::Database::connect(opt).await?;
    Migrator::up(&db, None).await?;
    Ok(db)
}
