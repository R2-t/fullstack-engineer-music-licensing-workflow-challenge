use crate::domain::*;
use crate::ports::AuditRepository;
use async_trait::async_trait;
use sqlx::{PgPool, Row};

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
    async fn log_transition(
        &self,
        track_id: i32,
        license_id: i32,
        user: String,
        old: Option<LicenseStatus>,
        new: LicenseStatus,
    ) -> Result<LicenseAuditEntry, DomainError> {
        let old_str = old.map(|s| s.to_string());
        let new_str = new.to_string();

        let row = sqlx::query("INSERT INTO license_audit_log (track_id, license_id, user_email, old_status, new_status) VALUES ($1, $2, $3, $4, $5) RETURNING id, track_id, license_id, user_email, old_status, new_status, changed_at")
            .bind(track_id)
            .bind(license_id)
            .bind(&user)
            .bind(&old_str)
            .bind(&new_str)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error logging audit for license {}: {}", license_id, e);
                DomainError::Internal
            })?;

        let old_status: Option<String> = row.get("old_status");
        let new_status: String = row.get("new_status");

        Ok(LicenseAuditEntry {
            id: row.get("id"),
            track_id: row.get("track_id"),
            license_id: row.get("license_id"),
            user_email: row.get("user_email"),
            old_status: old_status.and_then(|s| s.parse().ok()),
            new_status: new_status.parse().unwrap_or(LicenseStatus::Draft),
            changed_at: row.get("changed_at"),
        })
    }

    async fn list_by_license(
        &self,
        license_id: i32,
    ) -> Result<Vec<LicenseAuditEntry>, DomainError> {
        let rows = sqlx::query("SELECT id, track_id, license_id, user_email, old_status, new_status, changed_at FROM license_audit_log WHERE license_id = $1 ORDER BY changed_at ASC")
            .bind(license_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error listing audit for license {}: {}", license_id, e);
                DomainError::Internal
            })?;

        let entries = rows
            .iter()
            .map(|row| {
                let old_status: Option<String> = row.get("old_status");
                let new_status: String = row.get("new_status");
                LicenseAuditEntry {
                    id: row.get("id"),
                    track_id: row.get("track_id"),
                    license_id: row.get("license_id"),
                    user_email: row.get("user_email"),
                    old_status: old_status.and_then(|s| s.parse().ok()),
                    new_status: new_status.parse().unwrap_or(LicenseStatus::Draft),
                    changed_at: row.get("changed_at"),
                }
            })
            .collect();

        Ok(entries)
    }

    async fn find_after_id(
        &self,
        license_id: i32,
        after_id: i32,
    ) -> Result<Vec<LicenseAuditEntry>, DomainError> {
        let rows = sqlx::query("SELECT id, track_id, license_id, user_email, old_status, new_status, changed_at FROM license_audit_log WHERE license_id = $1 AND id > $2 ORDER BY id ASC")
            .bind(license_id)
            .bind(after_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("DB error finding audit after {} for license {}: {}", after_id, license_id, e);
                DomainError::Internal
            })?;

        let entries = rows
            .iter()
            .map(|row| {
                let old_status: Option<String> = row.get("old_status");
                let new_status: String = row.get("new_status");
                LicenseAuditEntry {
                    id: row.get("id"),
                    track_id: row.get("track_id"),
                    license_id: row.get("license_id"),
                    user_email: row.get("user_email"),
                    old_status: old_status.and_then(|s| s.parse().ok()),
                    new_status: new_status.parse().unwrap_or(LicenseStatus::Draft),
                    changed_at: row.get("changed_at"),
                }
            })
            .collect();

        Ok(entries)
    }
}
