use crate::domain::*;
use crate::ports::MovieRepository;
use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::{PgPool, Row};

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
        let rows = sqlx::query("SELECT id, title, release_date, created_at FROM movies ORDER BY created_at DESC LIMIT $1 OFFSET $2")
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error listing movies: {}", e);
                DomainError::Internal
            })?;

        let mut movies = Vec::new();
        for row in rows {
            movies.push(Movie {
                id: row.get("id"),
                title: row.get("title"),
                release_date: row.get("release_date"),
                created_at: row.get("created_at"),
            });
        }
        Ok(movies)
    }

    async fn find_by_id(&self, id: i32) -> Result<Movie, DomainError> {
        let row =
            sqlx::query("SELECT id, title, release_date, created_at FROM movies WHERE id = $1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| {
                    tracing::error!("DB error finding movie {}: {}", id, e);
                    DomainError::Internal
                })?
                .ok_or(DomainError::NotFound(format!("Movie {}", id)))?;

        Ok(Movie {
            id: row.get("id"),
            title: row.get("title"),
            release_date: row.get("release_date"),
            created_at: row.get("created_at"),
        })
    }

    async fn create(
        &self,
        title: String,
        release_date: Option<NaiveDate>,
    ) -> Result<Movie, DomainError> {
        let row = sqlx::query("INSERT INTO movies (title, release_date) VALUES ($1, $2) RETURNING id, title, release_date, created_at")
            .bind(&title)
            .bind(release_date)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error creating movie: {}", e);
                DomainError::Internal
            })?;

        Ok(Movie {
            id: row.get("id"),
            title: row.get("title"),
            release_date: row.get("release_date"),
            created_at: row.get("created_at"),
        })
    }
}
