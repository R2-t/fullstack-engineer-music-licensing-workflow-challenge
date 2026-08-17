use crate::domain::*;
use crate::ports::*;
use std::sync::Arc;

pub struct LicenseService<L, A, E> {
    license_repo: Arc<L>,
    audit_repo: Arc<A>,
    publisher: Arc<E>,
}

impl<L, A, E> LicenseService<L, A, E>
where
    L: LicenseRepository,
    A: AuditRepository,
    E: EventPublisher,
{
    pub fn new(license_repo: Arc<L>, audit_repo: Arc<A>, publisher: Arc<E>) -> Self {
        Self {
            license_repo,
            audit_repo,
            publisher,
        }
    }

    pub async fn initiate(
        &self,
        track_id: i32,
        label: Option<String>,
        artist: Option<String>,
        notes: serde_json::Value,
    ) -> Result<License, DomainError> {
        if self.license_repo.find_by_track(track_id).await?.is_some() {
            return Err(DomainError::ValidationError(
                "License already exists for this track".into(),
            ));
        }

        let license = self
            .license_repo
            .create(track_id, label, artist, notes)
            .await?;

        self.audit_repo
            .log_transition(
                track_id,
                license.id,
                "system".to_string(),
                None,
                LicenseStatus::Draft,
            )
            .await?;

        Ok(license)
    }

    pub async fn transition(
        &self,
        track_id: i32,
        target: LicenseStatus,
        user: String,
        notes: serde_json::Value,
    ) -> Result<License, DomainError> {
        let license =
            self.license_repo
                .find_by_track(track_id)
                .await?
                .ok_or(DomainError::NotFound(format!(
                    "License for track {}",
                    track_id
                )))?;

        crate::domain::workflow::WorkflowValidator::validate_transition(license.status, target)?;

        let updated = self
            .license_repo
            .update_status(license.id, target, notes)
            .await?;

        self.audit_repo
            .log_transition(
                track_id,
                license.id,
                user.clone(),
                Some(license.status),
                target,
            )
            .await?;

        self.publisher
            .publish_status_changed(LicenseEvent {
                track_id,
                license_id: license.id,
                previous_status: Some(license.status),
                status: target,
                changed_by: user,
                timestamp: chrono::Utc::now(),
            })
            .await;

        Ok(updated)
    }

    pub async fn get_status(&self, track_id: i32) -> Result<License, DomainError> {
        self.license_repo
            .find_by_track(track_id)
            .await?
            .ok_or(DomainError::NotFound(format!(
                "License for track {}",
                track_id
            )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::{MockAuditRepository, MockEventPublisher, MockLicenseRepository};
    use mockall::predicate::*;

    fn make_license(id: i32, track_id: i32, status: LicenseStatus) -> License {
        License {
            id,
            track_id,
            label_name: None,
            artist_name: None,
            status,
            negotiation_notes: serde_json::json!({}),
            last_updated_at: chrono::Utc::now(),
        }
    }

    fn make_audit_entry(id: i32, license_id: i32, track_id: i32) -> LicenseAuditEntry {
        LicenseAuditEntry {
            id,
            track_id,
            license_id,
            user_email: "system".to_string(),
            old_status: None,
            new_status: LicenseStatus::Draft,
            changed_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn initiate_creates_license_when_none_exists() {
        let mut license_repo = MockLicenseRepository::new();
        let mut audit_repo = MockAuditRepository::new();
        let mut publisher = MockEventPublisher::new();

        license_repo
            .expect_find_by_track()
            .with(eq(1))
            .returning(|_| Ok(None));

        license_repo
            .expect_create()
            .with(eq(1), eq(None), eq(None), always())
            .returning(|track_id, _, _, _| Ok(make_license(1, track_id, LicenseStatus::Draft)));

        audit_repo
            .expect_log_transition()
            .returning(|_, _, _, _, _| Ok(make_audit_entry(1, 1, 1)));

        let service = LicenseService::new(
            Arc::new(license_repo),
            Arc::new(audit_repo),
            Arc::new(publisher),
        );

        let result = service.initiate(1, None, None, serde_json::json!({})).await;
        assert!(result.is_ok());
        let license = result.unwrap();
        assert_eq!(license.track_id, 1);
        assert_eq!(license.status, LicenseStatus::Draft);
    }

    #[tokio::test]
    async fn initiate_rejects_duplicate_license() {
        let mut license_repo = MockLicenseRepository::new();
        let audit_repo = MockAuditRepository::new();
        let mut publisher = MockEventPublisher::new();

        license_repo
            .expect_find_by_track()
            .with(eq(1))
            .returning(|_| Ok(Some(make_license(1, 1, LicenseStatus::Draft))));

        let service = LicenseService::new(
            Arc::new(license_repo),
            Arc::new(audit_repo),
            Arc::new(publisher),
        );

        let result = service.initiate(1, None, None, serde_json::json!({})).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::ValidationError(msg) => assert!(msg.contains("already exists")),
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn transition_draft_to_negotiating_succeeds() {
        let mut license_repo = MockLicenseRepository::new();
        let mut audit_repo = MockAuditRepository::new();
        let mut publisher = MockEventPublisher::new();

        license_repo
            .expect_find_by_track()
            .with(eq(1))
            .returning(|_| Ok(Some(make_license(1, 1, LicenseStatus::Draft))));

        license_repo
            .expect_update_status()
            .with(eq(1), eq(LicenseStatus::Negotiating), always())
            .returning(|id, status, _| Ok(make_license(id, 1, status)));

        audit_repo
            .expect_log_transition()
            .returning(|_, _, _, _, _| Ok(make_audit_entry(1, 1, 1)));

        publisher.expect_publish_status_changed().returning(|_| ());

        let service = LicenseService::new(
            Arc::new(license_repo),
            Arc::new(audit_repo),
            Arc::new(publisher),
        );

        let result = service
            .transition(
                1,
                LicenseStatus::Negotiating,
                "user@test.com".to_string(),
                serde_json::json!({}),
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn transition_draft_to_approved_fails() {
        let mut license_repo = MockLicenseRepository::new();
        let audit_repo = MockAuditRepository::new();
        let mut publisher = MockEventPublisher::new();

        license_repo
            .expect_find_by_track()
            .with(eq(1))
            .returning(|_| Ok(Some(make_license(1, 1, LicenseStatus::Draft))));

        let service = LicenseService::new(
            Arc::new(license_repo),
            Arc::new(audit_repo),
            Arc::new(publisher),
        );

        let result = service
            .transition(
                1,
                LicenseStatus::Approved,
                "user@test.com".to_string(),
                serde_json::json!({}),
            )
            .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::InvalidTransition { from, to } => {
                assert_eq!(from, LicenseStatus::Draft);
                assert_eq!(to, LicenseStatus::Approved);
            }
            _ => panic!("Expected InvalidTransition"),
        }
    }

    #[tokio::test]
    async fn transition_approved_to_anything_fails() {
        for target in [
            LicenseStatus::Draft,
            LicenseStatus::Negotiating,
            LicenseStatus::Rejected,
        ] {
            let mut license_repo = MockLicenseRepository::new();
            let audit_repo = MockAuditRepository::new();
            let mut publisher = MockEventPublisher::new();

            license_repo
                .expect_find_by_track()
                .returning(|_| Ok(Some(make_license(1, 1, LicenseStatus::Approved))));

            let service = LicenseService::new(
                Arc::new(license_repo),
                Arc::new(audit_repo),
                Arc::new(publisher),
            );

            let result = service
                .transition(
                    1,
                    target,
                    "user@test.com".to_string(),
                    serde_json::json!({}),
                )
                .await;
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn transition_not_found_license_returns_error() {
        let mut license_repo = MockLicenseRepository::new();
        let audit_repo = MockAuditRepository::new();
        let mut publisher = MockEventPublisher::new();

        license_repo.expect_find_by_track().returning(|_| Ok(None));

        let service = LicenseService::new(
            Arc::new(license_repo),
            Arc::new(audit_repo),
            Arc::new(publisher),
        );

        let result = service
            .transition(
                999,
                LicenseStatus::Negotiating,
                "user@test.com".to_string(),
                serde_json::json!({}),
            )
            .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::NotFound(msg) => assert!(msg.contains("999")),
            _ => panic!("Expected NotFound"),
        }
    }

    #[tokio::test]
    async fn get_status_returns_license() {
        let mut license_repo = MockLicenseRepository::new();
        let audit_repo = MockAuditRepository::new();
        let mut publisher = MockEventPublisher::new();

        license_repo
            .expect_find_by_track()
            .with(eq(1))
            .returning(|_| Ok(Some(make_license(1, 1, LicenseStatus::Negotiating))));

        let service = LicenseService::new(
            Arc::new(license_repo),
            Arc::new(audit_repo),
            Arc::new(publisher),
        );

        let result = service.get_status(1).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, LicenseStatus::Negotiating);
    }

    #[tokio::test]
    async fn get_status_not_found() {
        let mut license_repo = MockLicenseRepository::new();
        let audit_repo = MockAuditRepository::new();
        let mut publisher = MockEventPublisher::new();

        license_repo.expect_find_by_track().returning(|_| Ok(None));

        let service = LicenseService::new(
            Arc::new(license_repo),
            Arc::new(audit_repo),
            Arc::new(publisher),
        );

        let result = service.get_status(999).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn initiate_then_transition_draft_to_negotiating() {
        let mut license_repo = MockLicenseRepository::new();
        let mut audit_repo = MockAuditRepository::new();
        let mut publisher = MockEventPublisher::new();

        license_repo
            .expect_find_by_track()
            .times(1)
            .returning(|_| Ok(None));
        license_repo
            .expect_create()
            .times(1)
            .returning(|track_id, _, _, _| Ok(make_license(1, track_id, LicenseStatus::Draft)));
        audit_repo
            .expect_log_transition()
            .times(1)
            .returning(|_, _, _, _, _| Ok(make_audit_entry(1, 1, 1)));

        license_repo
            .expect_find_by_track()
            .times(1)
            .returning(|_| Ok(Some(make_license(1, 1, LicenseStatus::Draft))));
        license_repo
            .expect_update_status()
            .times(1)
            .returning(|id, status, _| Ok(make_license(id, 1, status)));
        audit_repo
            .expect_log_transition()
            .times(1)
            .returning(|_, _, _, _, _| Ok(make_audit_entry(2, 1, 1)));
        publisher
            .expect_publish_status_changed()
            .times(1)
            .returning(|_| ());

        let service = LicenseService::new(
            Arc::new(license_repo),
            Arc::new(audit_repo),
            Arc::new(publisher),
        );

        let license = service
            .initiate(1, None, None, serde_json::json!({}))
            .await
            .unwrap();
        assert_eq!(license.status, LicenseStatus::Draft);

        let license = service
            .transition(
                1,
                LicenseStatus::Negotiating,
                "user@test.com".to_string(),
                serde_json::json!({}),
            )
            .await
            .unwrap();
        assert_eq!(license.status, LicenseStatus::Negotiating);
    }
}
