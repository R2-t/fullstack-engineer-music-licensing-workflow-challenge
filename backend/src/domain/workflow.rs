use crate::domain::{LicenseStatus, DomainError};

pub struct WorkflowValidator;

impl WorkflowValidator {
    pub fn validate_transition(from: LicenseStatus, to: LicenseStatus) -> Result<(), DomainError> {
        if from == to {
            return Ok(());
        }
        
        let valid = match (from, to) {
            (LicenseStatus::Draft, LicenseStatus::Negotiating) => true,
            (LicenseStatus::Negotiating, LicenseStatus::Approved) => true,
            (LicenseStatus::Negotiating, LicenseStatus::Rejected) => true,
            (LicenseStatus::Rejected, LicenseStatus::Negotiating) => true,
            _ => false,
        };

        if valid {
            Ok(())
        } else {
            Err(DomainError::InvalidTransition { from, to })
        }
    }
}
