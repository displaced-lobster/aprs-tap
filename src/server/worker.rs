use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};

use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serde::Serialize;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::Notify,
};
use uuid::Uuid;

use super::entities::{position, user};
use crate::aprs::{Altitude, AprsPacket};

pub type StatusHandle = Arc<RwLock<WorkerStatus>>;

#[derive(Clone, Debug, Serialize, utoipa::ToSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum WorkerStatus {
    Waiting,
    Listening,
    Error { message: String },
}

pub struct WorkerConfig {
    pub aprs_server: String,
    pub aprs_port: u16,
    pub callsign: String,
    pub passcode: String,
}

fn set_status(handle: &StatusHandle, status: WorkerStatus) {
    *handle.write().unwrap() = status;
}

pub async fn run(
    db: DatabaseConnection,
    config: WorkerConfig,
    notify: Arc<Notify>,
    status: StatusHandle,
) {
    loop {
        let users = load_users(&db).await;
        if users.is_empty() {
            set_status(&status, WorkerStatus::Waiting);
            notify.notified().await;
            continue;
        }

        let filter = format!("p/{}", users.keys().cloned().collect::<Vec<_>>().join("/"));

        tokio::select! {
            result = connect_and_stream(&db, &config, &filter, &users, &status) => {
                if let Err(e) = result {
                    set_status(&status, WorkerStatus::Error { message: e.to_string() });
                    eprintln!("aprs stream error: {e}");
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
            _ = notify.notified() => {}
        }
    }
}

// Returns HashMap<uppercase_base_callsign, user_id>
async fn load_users(db: &DatabaseConnection) -> HashMap<String, String> {
    match user::Entity::find().all(db).await {
        Ok(rows) => rows.into_iter().map(|u| (u.callsign, u.id)).collect(),
        Err(e) => {
            eprintln!("failed to load users: {e}");
            HashMap::new()
        }
    }
}

async fn connect_and_stream(
    db: &DatabaseConnection,
    config: &WorkerConfig,
    filter: &str,
    users: &HashMap<String, String>,
    status: &StatusHandle,
) -> std::io::Result<()> {
    let stream = TcpStream::connect(format!("{}:{}", config.aprs_server, config.aprs_port)).await?;
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);

    let login = format!(
        "user {} pass {} vers aprs-tap 0.1.0 filter {}\r\n",
        config.callsign, config.passcode, filter
    );
    write_half.write_all(login.as_bytes()).await?;
    set_status(status, WorkerStatus::Listening);

    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }
        let trimmed = line.trim_end();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Err(e) = handle_line(db, trimmed, users).await {
            eprintln!("position insert failed: {e}");
        }
    }
    Ok(())
}

async fn handle_line(
    db: &DatabaseConnection,
    line: &str,
    users: &HashMap<String, String>,
) -> Result<(), sea_orm::DbErr> {
    let Ok(packet) = AprsPacket::try_from(line) else {
        return Ok(());
    };
    let Some(pos) = packet.try_position() else {
        return Ok(());
    };

    let base = packet
        .source()
        .split('-')
        .next()
        .unwrap_or_else(|| packet.source())
        .to_uppercase();

    let Some(user_id) = users.get(&base) else {
        return Ok(());
    };

    let alt = Altitude::from(pos.comment);
    let altitude_m = alt.meters();
    let comment = (!alt.comment.is_empty()).then_some(alt.comment);

    position::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        user_id: Set(user_id.clone()),
        latitude: Set(pos.lat.0),
        longitude: Set(pos.lon.0),
        altitude_m: Set(altitude_m),
        symbol_table: Set(pos.sym_table.to_string()),
        symbol_code: Set(pos.sym_code.to_string()),
        comment: Set(comment),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(())
}
