use clap::Parser;
use colored::Colorize;
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

#[derive(Parser)]
#[command(name = "aprs-server", about = "APRS-IS HTTP server")]
struct Args {
    /// Address to bind the HTTP server on
    #[arg(long, env = "BIND_ADDR", default_value = "0.0.0.0")]
    host: String,

    /// Port to listen on
    #[arg(short, long, env = "PORT", default_value_t = 3000)]
    port: u16,

    /// APRS-IS server hostname
    #[arg(long, env = "APRS_SERVER", default_value = "rotate.aprs2.net")]
    aprs_server: String,

    /// APRS-IS server port
    #[arg(long, env = "APRS_PORT", default_value_t = 14580)]
    aprs_port: u16,

    /// Your callsign (N0CALL for anonymous read-only)
    #[arg(short = 'u', long, env = "APRS_CALLSIGN", default_value = "N0CALL")]
    callsign: String,

    /// APRS-IS passcode (-1 for read-only)
    #[arg(long, env = "APRS_PASSCODE", default_value = "-1")]
    passcode: String,

    /// Server-side filter string, e.g. "r/38.9/-77.0/100"
    #[arg(short, long, env = "APRS_FILTER")]
    filter: Option<String>,

    /// Database connection URL (sqlite://./dev.db or postgres://user:pass@host/db)
    #[arg(long, env = "DATABASE_URL", default_value = "sqlite://./dev.db")]
    database_url: String,

    /// JWT signing secret
    #[arg(long, env = "JWT_SECRET")]
    jwt_secret: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let args = Args::parse();

    print_banner();
    print_config(&args);

    let db = aprs_tap::server::db::connect(&args.database_url)
        .await
        .expect("failed to connect to database");

    let worker_notify = Arc::new(tokio::sync::Notify::new());
    let worker_status = Arc::new(RwLock::new(aprs_tap::server::worker::WorkerStatus::Waiting));

    let state = aprs_tap::server::db::AppState {
        db: db.clone(),
        jwt_secret: args.jwt_secret.clone(),
        worker_notify: Arc::clone(&worker_notify),
        worker_status: Arc::clone(&worker_status),
    };

    let worker_config = aprs_tap::server::worker::WorkerConfig {
        aprs_server: args.aprs_server.clone(),
        aprs_port: args.aprs_port,
        callsign: args.callsign.clone(),
        passcode: args.passcode.clone(),
    };
    tokio::spawn(aprs_tap::server::worker::run(
        db,
        worker_config,
        worker_notify,
        worker_status,
    ));

    let addr: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .expect("invalid bind address");

    let app = aprs_tap::server::routes::router(state);

    println!(
        "\n{} listening on {}\n",
        "aprs-server".green().bold(),
        format!("http://{addr}").cyan().underline(),
    );

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn print_banner() {
    println!("{}", "┌─────────────────────────────┐".dimmed());
    println!("{}", "│       aprs-tap  server      │".dimmed());
    println!("{}", "└─────────────────────────────┘".dimmed());
    println!();
}

fn print_config(args: &Args) {
    let label = |s: &str| s.bold().to_string();
    let val = |s: &str| s.cyan().to_string();

    println!(
        "  {}  {}",
        label("http bind"),
        val(&format!("{}:{}", args.host, args.port))
    );
    println!(
        "  {}  {}",
        label("aprs feed"),
        val(&format!("{}:{}", args.aprs_server, args.aprs_port))
    );
    println!("  {}   {}", label("callsign"), val(&args.callsign));
    println!("  {}   {}", label("passcode"), val(&args.passcode));
    if let Some(f) = &args.filter {
        println!("  {}     {}", label("filter"), val(f));
    }
    println!("  {}    {}", label("database"), val(&args.database_url));
    println!("  {}  {}", label("jwt secret"), val("[set]"));
}
