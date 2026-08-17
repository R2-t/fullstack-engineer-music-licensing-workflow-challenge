use std::sync::Arc;
use crate::domain::*;
use crate::ports::*;
use async_trait::async_trait;

pub struct LicenseService<L, A, E> {
    license_repo: Arc<L>,
    audit_repo: Arc<A>,
    publisher: Arc<E>,
}

impl<L, A, E> LicenseService<L, A, E> 
where 
    L: LicenseRepository, 
    A: AuditRepository, 
    E: EventPublisher 
{
    pub fn new(license_repo: Arc<L>, audit_repo: Arc<A>, publisher: Arc<E>) -> Self {
        Self { license_repo, audit_repo, publisher }
    }

    pub async fn initiate(&self, track_id: i32, label: Option<String>, artist: Option<String>, notes: serde_json::Value) -> Result<License, DomainError> {
        if let Some(_) = self.license_repo.find_by_track(track_id).await? {
            return Err(DomainError::ValidationError("License already exists for this track".into()));
        }
        
        let license = self.license_repo.create(track_id, label, artist, notes).await?;
        
        self.audit_repo.log_transition(track_id, license.id, "system".to_string(), None, LicenseStatus::Draft).await?;
        
        Ok(license)
    }

    pub async fn transition(&self, track_id: i32, target: LicenseStatus, user: String, notes: serde_json::Value) -> Result<License, DomainError> {
        let license = self.license_repo.find_by_track(track_id).await?
            .ok_or(DomainError::NotFound(format!("License for track {}", track_id)))?;
        
        crate::domain::workflow::WorkflowValidator::validate_transition(license.status, target)?;
        
        let updated = self.license_repo.update_status(license.id, target, notes).await?;
        
        self.audit_repo.log_transition(track_id, license.id, user, Some(license.status), target).await?;
        
        self.publisher.publish_status_changed(LicenseEvent {
            track_id,
            license_id: license.id,
            previous_status: Some(license.status),
            status: target,
            changed_by: user,
            timestamp: chrono::Utc::now(),
        }).await;

        Ok(updated)
    }

    pub async fn get_status(&self, track_id: i32) -> Result<License, DomainError> {
        self.license_repo.find_by_track(track_id).await?
            .ok_or(DomainError::NotFound(format!("License for track {}", track_id)))
    }
}
