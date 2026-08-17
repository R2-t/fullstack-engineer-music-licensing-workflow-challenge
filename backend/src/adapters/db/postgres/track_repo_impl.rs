use sqlx::{PgPool};
use crate::domain::*;
use crate::ports::TrackRepository;
use async_trait::async_trait;

pub struct PostgresTrackRepository {
    pool: PgPool,
}

impl PostgresTrackRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TrackRepository for PostgresTrackRepository {
    async fn list_by_scene(&self, scene_id: i32, page: i32, limit: i32) -> Result<Vec<Track>, DomainError> {
        let offset = (page - 1) * limit;
        let rows = sqlx::query!(
            "SELECT id, scene_id, track_order, name, song_title, song_artist, song_label, duration_sec_start, duration_sec_end, created_at FROM tracks WHERE scene_id = $1 ORDER BY track_order ASC LIMIT $2 OFFSET $3",
            scene_id,
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error listing tracks for scene {}: {}", scene_id, e);
            DomainError::Internal
        })?;

        let tracks = rows.into_iter().map(|r| Track {
            id: r.id,
            scene_id: r.scene_id,
            track_order: r.track_order,
            name: r.name.unwrap_or_default(),
            song: Song {
                title: r.song_title.unwrap_or_default(),
                artist: r.song_artist,
                label_name: r.song_label,
                duration_sec_start: r.duration_sec_start,
                duration_sec_end: r.duration_sec_end,
            },
            created_at: r.created_at,
        }).collect();

        Ok(tracks)
    }

    async fn find_by_id(&self, id: i32) -> Result<Track, DomainError> {
        let r = sqlx::query!(
            "SELECT id, scene_id, track_order, name, song_title, song_artist, song_label, duration_sec_start, duration_sec_end, created_at FROM tracks WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error finding track {}: {}", id, e);
            DomainError::Internal
        })?
        .ok_or(DomainError::NotFound(format!("Track {}", id)))?;

        Ok(Track {
            id: r.id,
            scene_id: r.scene_id,
            track_order: r.track_order,
            name: r.name.unwrap_or_default(),
            song: Song {
                title: r.song_title.unwrap_or_default(),
                artist: r.song_artist,
                label_name: r.song_label,
                duration_sec_start: r.duration_sec_start,
                duration_sec_end: r.duration_sec_end,
            },
            created_at: r.created_at,
        })
    }

    async fn create(&self, scene_id: i32, order: i16, name: String, song: Song) -> Result<Track, DomainError> {
        let r = sqlx::query!(
            "INSERT INTO tracks (scene_id, track_order, name, song_title, song_artist, song_label, duration_sec_start, duration_sec_end) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
             RETURNING id, scene_id, track_order, name, song_title, song_artist, song_label, duration_sec_start, duration_sec_end, created_at",
            scene_id, order, name, song.title, song.artist, song.label_name, song.duration_sec_start, song.duration_sec_end
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Internal)?;

        Ok(Track {
            id: r.id,
            scene_id: r.scene_id,
            track_order: r.track_order,
            name: r.name.unwrap_or_default(),
            song: Song {
                title: r.song_title.unwrap_or_default(),
                artist: r.song_artist,
                label_name: r.song_label,
                duration_sec_start: r.duration_sec_start,
                duration_sec_end: r.duration_sec_end,
            },
            created_at: r.created_at,
        })
    }

    async fn update(&self, id: i32, name: Option<String>, order: Option<i16>, song: Option<Song>) -> Result<Track, DomainError> {
        // Simple update implementation for brevity, updating all fields if present
        let r = sqlx::query!(
            "UPDATE tracks SET name = COALESCE($2, name), track_order = COALESCE($3, track_order), 
             song_title = COALESCE($4, song_title), song_artist = COALESCE($5, song_artist), 
             song_label = COALESCE($6, song_label), duration_sec_start = COALESCE($7, duration_sec_start), 
             duration_sec_end = COALESCE($8, duration_sec_end) 
             WHERE id = $1 RETURNING id, scene_id, track_order, name, song_title, song_artist, song_label, duration_sec_start, duration_sec_end, created_at",
            id, name, order, song.as_ref().map(|s| &s.title), song.as_ref().map(|s| &s.artist), 
            song.as_ref().map(|s| &s.label_name), song.as_ref().map(|s| &s.duration_sec_start), song.as_ref().map(|s| &s.duration_sec_end)
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Internal)?;

        Ok(Track {
            id: r.id,
            scene_id: r.scene_id,
            track_order: r.track_order,
            name: r.name.unwrap_or_default(),
            song: Song {
                title: r.song_title.unwrap_or_default(),
                artist: r.song_artist,
                label_name: r.song_label,
                duration_sec_start: r.duration_sec_start,
                duration_sec_end: r.duration_sec_end,
            },
            created_at: r.created_at,
        })
    }

    async fn delete(&self, id: i32) -> Result<(), DomainError> {
        sqlx::query!("DELETE FROM tracks WHERE id = $1", id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error deleting track {}: {}", id, e);
            DomainError::Internal
        })?;
        Ok(())
    }
}
