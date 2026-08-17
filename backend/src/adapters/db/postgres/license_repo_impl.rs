use sqlx::{PgPool};
use crate::domain::*;
use crate::ports::LicenseRepository;
use async_trait::async_trait;

pub struct PostgresLicenseRepository {
    pool: PgPool,
}

impl PostgresLicenseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LicenseRepository for PostgresLicenseRepository {
    async fn find_by_track(&self, track_id: i32) -> Result<Option<License>, DomainError> {
        sqlx::query_as!(
            License,
            "SELECT id, track_id, label_name, artist_name, status as \"status: String\", negotiation_notes, last_updated_at FROM licenses WHERE track_id = $1",
            track_id
        )
        .fetch_optional(&self.pool)
        .await
        .map(|opt| opt.map(|r| License {
            id: r.id,
            track_id: r.track_id,
            label_name: r.label_name,
            artist_name: r.artist_name,
            status: r.status.parse().unwrap_or(LicenseStatus::Draft),
            negotiation_notes: r.negotiation_notes,
            last_updated_at: r.last_updated_at,
        }))
        .map_err(|e| DomainError::Internal)
    }

    async fn find_by_id(&self, id: i32) -> Result<License, DomainError> {
        let r = sqlx::query!(
            "SELECT id, track_id, label_name, artist_name, status, negotiation_notes, last_updated_at FROM licenses WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal)?
        .ok_or(DomainError::NotFound(format!("License {}", id)))?;

        Ok(License {
            id: r.id,
            track_id: r.track_id,
            label_name: r.label_name,
            artist_name: r.artist_name,
            status: r.status.parse().unwrap_or(LicenseStatus::Draft),
            negotiation_notes: r.negotiation_notes,
            last_updated_at: r.last_updated_at,
        })
    }

    async fn create(&self, track_id: i32, label: Option<String>, artist: Option<String>, notes: serde_json::Value) -> Result<License, DomainError> {
        let r = sqlx::query!(
            "INSERT INTO licenses (track_id, label_name, artist_name, status, negotiation_notes) VALUES ($1, $2, $3, 'DRAFT', $4) RETURNING id, track_id, label_name, artist_name, status, negotiation_notes, last_updated_at",
            track_id, label, artist, notes
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Internal)?;

        Ok(License {
            id: r.id,
            track_id: r.track_id,
            label_name: r.label_name,
            artist_name: r.artist_name,
            status: r.status.parse().unwrap_or(LicenseStatus::Draft),
            negotiation_notes: r.negotiation_notes,
            last_updated_at: r.last_updated_at,
        })
    }

    async fn update_status(&self, id: i32, status: LicenseStatus, notes: serde_json::Value) -> Result<License, DomainError> {
        let status_str = status.to_string();
        let r = sqlx::query!(
            "UPDATE licenses SET status = $1, negotiation_notes = negotiation_notes || $2, last_updated_at = CURRENT_TIMESTAMP WHERE id = $3 RETURNING id, track_id, label_name, artist_name, status, negotiation_notes, last_updated_at",
            status_str, notes, id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Internal)?;

        Ok(License {
            id: r.id,
            track_id: r.track_id,
            label_name: r.label_name,
            artist_name: r.artist_name,
            status: r.status.parse().unwrap_or(LicenseStatus::Draft),
            negotiation_notes: r.negotiation_notes,
            last_updated_at: r.last_updated_at,
        })
    }
}
