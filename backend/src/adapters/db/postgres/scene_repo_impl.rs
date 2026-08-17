use sqlx::{PgPool, Row};
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
        let rows = sqlx::query("SELECT id, movie_id, scene_number, created_at FROM scenes WHERE movie_id = $1 ORDER BY scene_number ASC LIMIT $2 OFFSET $3")
            .bind(movie_id)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error listing scenes for movie {}: {}", movie_id, e);
                DomainError::Internal
            })?;
        
        let mut scenes = Vec::new();
        for row in rows {
            scenes.push(Scene {
                id: row.get("id"),
                movie_id: row.get("movie_id"),
                scene_number: row.get("scene_number"),
                created_at: row.get("created_at"),
            });
        }
        Ok(scenes)
    }

    async fn find_by_id(&self, id: i32) -> Result<Scene, DomainError> {
        let row = sqlx::query("SELECT id, movie_id, scene_number, created_at FROM scenes WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error finding scene {}: {}", id, e);
                DomainError::Internal
            })?
            .ok_or(DomainError::NotFound(format!("Scene {}", id)))?;
        
        Ok(Scene {
            id: row.get("id"),
            movie_id: row.get("movie_id"),
            scene_number: row.get("scene_number"),
            created_at: row.get("created_at"),
        })
    }

    async fn create(&self, movie_id: i32, scene_number: i16) -> Result<Scene, DomainError> {
        let row = sqlx::query("INSERT INTO scenes (movie_id, scene_number) VALUES ($1, $2) RETURNING id, movie_id, scene_number, created_at")
            .bind(movie_id)
            .bind(scene_number)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error creating scene for movie {}: {}", movie_id, e);
                DomainError::Internal
            })?;
        
        Ok(Scene {
            id: row.get("id"),
            movie_id: row.get("movie_id"),
            scene_number: row.get("scene_number"),
            created_at: row.get("created_at"),
        })
    }
}