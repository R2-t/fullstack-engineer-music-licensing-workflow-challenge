use crate::domain::*;
use crate::ports::LicenseRepository;
use async_trait::async_trait;
use sqlx::{PgPool, Row};

pub struct PostgresLicenseRepository {
    pool: PgPool,
}

impl PostgresLicenseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_license(row: &sqlx::postgres::PgRow) -> License {
    let status_str: String = row.get("status");
    License {
        id: row.get("id"),
        track_id: row.get("track_id"),
        label_name: row.get("label_name"),
        artist_name: row.get("artist_name"),
        status: status_str.parse().unwrap_or(LicenseStatus::Draft),
        negotiation_notes: row.get("negotiation_notes"),
        last_updated_at: row.get("last_updated_at"),
    }
}

#[async_trait]
impl LicenseRepository for PostgresLicenseRepository {
    #[tracing::instrument(skip(self), name = "db.license.find_by_track")]
    async fn find_by_track(&self, track_id: i32) -> Result<Option<License>, DomainError> {
        let row = sqlx::query("SELECT id, track_id, label_name, artist_name, status, negotiation_notes, last_updated_at FROM licenses WHERE track_id = $1")
            .bind(track_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error finding license for track {}: {}", track_id, e);
                DomainError::Internal
            })?;

        Ok(row.as_ref().map(row_to_license))
    }

    #[tracing::instrument(skip(self), name = "db.license.find_by_id")]
    async fn find_by_id(&self, id: i32) -> Result<License, DomainError> {
        let row = sqlx::query("SELECT id, track_id, label_name, artist_name, status, negotiation_notes, last_updated_at FROM licenses WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error finding license {}: {}", id, e);
                DomainError::Internal
            })?
            .ok_or(DomainError::NotFound(format!("License {}", id)))?;

        Ok(row_to_license(&row))
    }

    #[tracing::instrument(skip(self, notes), name = "db.license.create")]
    async fn create(
        &self,
        track_id: i32,
        label: Option<String>,
        artist: Option<String>,
        notes: serde_json::Value,
    ) -> Result<License, DomainError> {
        let row = sqlx::query("INSERT INTO licenses (track_id, label_name, artist_name, status, negotiation_notes) VALUES ($1, $2, $3, 'DRAFT', $4) RETURNING id, track_id, label_name, artist_name, status, negotiation_notes, last_updated_at")
            .bind(track_id)
            .bind(&label)
            .bind(&artist)
            .bind(&notes)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error creating license for track {}: {}", track_id, e);
                DomainError::Internal
            })?;

        Ok(row_to_license(&row))
    }

    #[tracing::instrument(skip(self, notes), name = "db.license.update_status")]
    async fn update_status(
        &self,
        id: i32,
        status: LicenseStatus,
        notes: serde_json::Value,
    ) -> Result<License, DomainError> {
        let status_str = status.to_string();
        let row = sqlx::query("UPDATE licenses SET status = $1, negotiation_notes = negotiation_notes || $2, last_updated_at = CURRENT_TIMESTAMP WHERE id = $3 RETURNING id, track_id, label_name, artist_name, status, negotiation_notes, last_updated_at")
            .bind(&status_str)
            .bind(&notes)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error updating license {}: {}", id, e);
                DomainError::Internal
            })?;

        Ok(row_to_license(&row))
    }
}
