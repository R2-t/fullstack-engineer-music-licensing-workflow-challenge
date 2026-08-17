pub mod domain;
pub mod application;
pub mod ports;
pub mod adapters;
pub mod config;
pub mod error;

use std::sync::Arc;
use axum::{Router, routing::{get, post, patch, delete}, Router as AxumRouter};
use crate::adapters::http::router::create_router;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config = config::Config::from_env();
    let pool = sqlx::PgPool::connect(&config.database_url).await.expect("Failed to connect to DB");
    
    sqlx::migrate!("./migrations").run(&pool).await.expect("Migration failed");

    let app = create_router(pool);
    
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port)).await.unwrap();
    tracing::info!("Server running on http://{}:{}", config.host, config.port);
    axum::serve(listener, app).await.unwrap();
}
