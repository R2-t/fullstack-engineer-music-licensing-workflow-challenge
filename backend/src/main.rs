use music_licensing_backend::adapters::http::router::create_router;
use music_licensing_backend::config;
use music_licensing_backend::infrastructure;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    infrastructure::tracing::init_tracing();

    let config = config::Config::from_env();
    let pool = sqlx::PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    let migration_sql = include_str!("../migrations/0001_init.sql");
    for statement in migration_sql.split(';') {
        let statement: String = statement
            .lines()
            .filter(|line| !line.trim_start().starts_with("--"))
            .collect::<Vec<_>>()
            .join(" ");
        let statement = statement.trim();
        if !statement.is_empty() {
            sqlx::query(statement)
                .execute(&pool)
                .await
                .expect("Failed to run migrations");
        }
    }

    let app = create_router(pool, config.jwt_secret.clone());

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port))
        .await
        .expect("Failed to bind listener");

    tracing::info!("Server running on http://{}:{}", config.host, config.port);
    axum::serve(listener, app).await.expect("Server error");
}
