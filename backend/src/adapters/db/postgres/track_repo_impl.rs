use crate::domain::*;
use crate::ports::TrackRepository;
use async_trait::async_trait;
use sqlx::{PgPool, Row};

pub struct PostgresTrackRepository {
    pool: PgPool,
}

impl PostgresTrackRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_track(row: &sqlx::postgres::PgRow) -> Track {
    Track {
        id: row.get("id"),
        scene_id: row.get("scene_id"),
        track_order: row.get("track_order"),
        name: row.get::<Option<String>, _>("name").unwrap_or_default(),
        song: Song {
            title: row
                .get::<Option<String>, _>("song_title")
                .unwrap_or_default(),
            artist: row.get("song_artist"),
            label_name: row.get("song_label"),
            duration_sec_start: row.get("duration_sec_start"),
            duration_sec_end: row.get("duration_sec_end"),
        },
        created_at: row.get("created_at"),
    }
}

#[async_trait]
impl TrackRepository for PostgresTrackRepository {
    async fn list_by_scene(
        &self,
        scene_id: i32,
        page: i32,
        limit: i32,
    ) -> Result<Vec<Track>, DomainError> {
        let offset = (page - 1) * limit;
        let rows = sqlx::query("SELECT id, scene_id, track_order, name, song_title, song_artist, song_label, duration_sec_start, duration_sec_end, created_at FROM tracks WHERE scene_id = $1 ORDER BY track_order ASC LIMIT $2 OFFSET $3")
            .bind(scene_id)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error listing tracks for scene {}: {}", scene_id, e);
                DomainError::Internal
            })?;

        Ok(rows.iter().map(row_to_track).collect())
    }

    async fn find_by_id(&self, id: i32) -> Result<Track, DomainError> {
        let row = sqlx::query("SELECT id, scene_id, track_order, name, song_title, song_artist, song_label, duration_sec_start, duration_sec_end, created_at FROM tracks WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error finding track {}: {}", id, e);
                DomainError::Internal
            })?
            .ok_or(DomainError::NotFound(format!("Track {}", id)))?;

        Ok(row_to_track(&row))
    }

    async fn create(
        &self,
        scene_id: i32,
        order: i16,
        name: String,
        song: Song,
    ) -> Result<Track, DomainError> {
        let row = sqlx::query("INSERT INTO tracks (scene_id, track_order, name, song_title, song_artist, song_label, duration_sec_start, duration_sec_end) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id, scene_id, track_order, name, song_title, song_artist, song_label, duration_sec_start, duration_sec_end, created_at")
            .bind(scene_id)
            .bind(order)
            .bind(&name)
            .bind(&song.title)
            .bind(&song.artist)
            .bind(&song.label_name)
            .bind(song.duration_sec_start)
            .bind(song.duration_sec_end)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error creating track in scene {}: {}", scene_id, e);
                DomainError::Internal
            })?;

        Ok(row_to_track(&row))
    }

    async fn update(
        &self,
        id: i32,
        name: Option<String>,
        order: Option<i16>,
        song: Option<Song>,
    ) -> Result<Track, DomainError> {
        let row = sqlx::query("UPDATE tracks SET name = COALESCE($2, name), track_order = COALESCE($3, track_order), song_title = COALESCE($4, song_title), song_artist = COALESCE($5, song_artist), song_label = COALESCE($6, song_label), duration_sec_start = COALESCE($7, duration_sec_start), duration_sec_end = COALESCE($8, duration_sec_end) WHERE id = $1 RETURNING id, scene_id, track_order, name, song_title, song_artist, song_label, duration_sec_start, duration_sec_end, created_at")
            .bind(id)
            .bind(&name)
            .bind(order)
            .bind(song.as_ref().map(|s| &s.title))
            .bind(song.as_ref().map(|s| &s.artist))
            .bind(song.as_ref().map(|s| &s.label_name))
            .bind(song.as_ref().map(|s| s.duration_sec_start))
            .bind(song.as_ref().map(|s| s.duration_sec_end))
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error updating track {}: {}", id, e);
                DomainError::Internal
            })?;

        Ok(row_to_track(&row))
    }

    async fn delete(&self, id: i32) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM tracks WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error deleting track {}: {}", id, e);
                DomainError::Internal
            })?;
        Ok(())
    }
}
