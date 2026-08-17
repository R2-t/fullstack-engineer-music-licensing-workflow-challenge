use sqlx::{PgPool, Postgres, Transaction};
use crate::domain::*;
use crate::ports::MovieRepository;
use async_trait::async_trait;
use chrono::NaiveDate;

pub struct PostgresMovieRepository {
    pool: PgPool,
}

impl PostgresMovieRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MovieRepository for PostgresMovieRepository {
    async fn list(&self, page: i32, limit: i32) -> Result<Vec<Movie>, DomainError> {
        let offset = (page - 1) * limit;
        let movies = sqlx::query_as!(
            Movie,
            "SELECT id, title, release_date, created_at FROM movies ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error listing movies: {}", e);
            DomainError::Internal
        })?;
        
        Ok(movies)
    }

    async fn find_by_id(&self, id: i32) -> Result<Movie, DomainError> {
        sqlx::query_as!(
            Movie,
            "SELECT id, title, release_date, created_at FROM movies WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error finding movie {}: {}", id, e);
            DomainError::Internal
        })?
        .ok_or(DomainError::NotFound(format!("Movie {}", id)))
    }

    async fn create(&self, title: String, release_date: Option<NaiveDate>) -> Result<Movie, DomainError> {
        sqlx::query_as!(
            Movie,
            "INSERT INTO movies (title, release_date) VALUES ($1, $2) RETURNING id, title, release_date, created_at",
            title,
            release_date
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error creating movie: {}", e);
            DomainError::Internal
        })
    }
}
