use sqlx::{PgPool};
use crate::domain::*;
use crate::ports::AuditRepository;
use async_trait::async_trait;

pub struct PostgresAuditRepository {
    pool: PgPool,
}

impl PostgresAuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditRepository for PostgresAuditRepository {
    async fn log_transition(&self, track_id: i32, license_id: i32, user: String, old: Option<LicenseStatus>, new: LicenseStatus) -> Result<LicenseAuditEntry, DomainError> {
        let old_str = old.map(|s| s.to_string());
        let new_str = new.to_string();
        
        let r = sqlx::query!(
            "INSERT INTO license_audit_log (track_id, license_id, user_email, old_status, new_status) VALUES ($1, $2, $3, $4, $5) RETURNING id, track_id, license_id, user_email, old_status, new_status, changed_at",
            track_id, license_id, user, old_str, new_str
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Internal)?;

        Ok(LicenseAuditEntry {
            id: r.id,
            track_id: r.track_id,
            license_id: r.license_id,
            user_email: r.user_email,
            old_status: r.old_status.map(|s| s.parse().unwrap_or(LicenseStatus::Draft)),
            new_status: r.new_status.parse().unwrap_or(LicenseStatus::Draft),
            changed_at: r.changed_at,
        })
    }

    async fn list_by_license(&self, license_id: i32) -> Result<Vec<LicenseAuditEntry>, DomainError> {
        let rows = sqlx::query!(
            "SELECT id, track_id, license_id, user_email, old_status, new_status, changed_at FROM license_audit_log WHERE license_id = $1 ORDER BY changed_at ASC",
            license_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal)?;

        let entries = rows.into_iter().map(|r| LicenseAuditEntry {
            id: r.id,
            track_id: r.track_id,
            license_id: r.license_id,
            user_email: r.user_email,
            old_status: r.old_status.map(|s| s.parse().unwrap_or(LicenseStatus::Draft)),
            new_status: r.new_status.parse().unwrap_or(LicenseStatus::Draft),
            changed_at: r.changed_at,
        }).collect();

        Ok(entries)
    }

    async fn find_after_id(&self, license_id: i32, after_id: i32) -> Result<Vec<LicenseAuditEntry>, DomainError> {
        let rows = sqlx::query!(
            "SELECT id, track_id, license_id, user_email, old_status, new_status, changed_at FROM license_audit_log WHERE license_id = $1 AND id > $2 ORDER BY id ASC",
            license_id, after_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal)?;

        let entries = rows.into_iter().map(|r| LicenseAuditEntry {
            id: r.id,
            track_id: r.track_id,
            license_id: r.license_id,
            user_email: r.user_email,
            old_status: r.old_status.map(|s| s.parse().unwrap_or(LicenseStatus::Draft)),
            new_status: r.new_status.parse().unwrap_or(LicenseStatus::Draft),
            changed_at: r.changed_at,
        }).collect();

        Ok(entries)
    }
}
