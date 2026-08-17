use sqlx::{PgPool, Postgres};
use crate::domain::*;
use crate::ports::SceneRepository;
use async_trait::async_trait;

pub struct PostgresSceneRepository {
    pool: PgPool,
}

impl PostgresSceneRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SceneRepository for PostgresSceneRepository {
    async fn list_by_movie(&self, movie_id: i32, page: i32, limit: i32) -> Result<Vec<Scene>, DomainError> {
        let offset = (page - 1) * limit;
        let scenes = sqlx::query_as!(
            Scene,
            "SELECT id, movie_id, scene_number, created_at FROM scenes WHERE movie_id = $1 ORDER BY scene_number ASC LIMIT $2 OFFSET $3",
            movie_id,
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error listing scenes for movie {}: {}", movie_id, e);
            DomainError::Internal
        })?;
        
        Ok(scenes)
    }

    async fn find_by_id(&self, id: i32) -> Result<Scene, DomainError> {
        sqlx::query_as!(
            Scene,
            "SELECT id, movie_id, scene_number, created_at FROM scenes WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error finding scene {}: {}", id, e);
            DomainError::Internal
        })?
        .ok_or(DomainError::NotFound(format!("Scene {}", id)))
    }

    async fn create(&self, movie_id: i32, scene_number: i16) -> Result<Scene, DomainError> {
        sqlx::query_as!(
            Scene,
            "INSERT INTO scenes (movie_id, scene_number) VALUES ($1, $2) RETURNING id, movie_id, scene_number, created_at",
            movie_id,
            scene_number
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error creating scene for movie {}: {}", movie_id, e);
            DomainError::Internal
        })
    }
}
