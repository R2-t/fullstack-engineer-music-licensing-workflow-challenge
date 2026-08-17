pub mod domain;
pub mod application;
pub mod ports;
pub mod adapters;
pub mod config;
pub mod error;
pub mod infrastructure;

use crate::adapters::http::router::create_router;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    crate::infrastructure::tracing::init_tracing();

    let config = config::Config::from_env();
    let pool = sqlx::PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");
    
    sqlx::query(include_str!("../migrations/0001_init.sql"))
        .execute(&pool)
        .await
        .expect("Failed to run migrations");

    let app = create_router(pool, config.jwt_secret.clone());
    
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port))
        .await
        .expect("Failed to bind listener");
    
    tracing::info!("Server running on http://{}:{}", config.host, config.port);
    axum::serve(listener, app)
        .await
        .expect("Server error");
}