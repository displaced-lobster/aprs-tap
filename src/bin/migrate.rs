use colored::Colorize;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://./dev.db".to_string());

    println!("  {}  {}", "database".bold(), database_url.cyan());

    aprs_tap::db::connect(&database_url)
        .await
        .expect("failed to connect to database");

    println!("{}", "migrations applied".green().bold());
}
