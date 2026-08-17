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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draft_to_negotiating_is_valid() {
        assert!(WorkflowValidator::validate_transition(LicenseStatus::Draft, LicenseStatus::Negotiating).is_ok());
    }

    #[test]
    fn negotiating_to_approved_is_valid() {
        assert!(WorkflowValidator::validate_transition(LicenseStatus::Negotiating, LicenseStatus::Approved).is_ok());
    }

    #[test]
    fn negotiating_to_rejected_is_valid() {
        assert!(WorkflowValidator::validate_transition(LicenseStatus::Negotiating, LicenseStatus::Rejected).is_ok());
    }

    #[test]
    fn rejected_to_negotiating_is_valid() {
        assert!(WorkflowValidator::validate_transition(LicenseStatus::Rejected, LicenseStatus::Negotiating).is_ok());
    }

    #[test]
    fn same_status_is_noop() {
        assert!(WorkflowValidator::validate_transition(LicenseStatus::Draft, LicenseStatus::Draft).is_ok());
        assert!(WorkflowValidator::validate_transition(LicenseStatus::Negotiating, LicenseStatus::Negotiating).is_ok());
        assert!(WorkflowValidator::validate_transition(LicenseStatus::Approved, LicenseStatus::Approved).is_ok());
        assert!(WorkflowValidator::validate_transition(LicenseStatus::Rejected, LicenseStatus::Rejected).is_ok());
    }

    #[test]
    fn draft_to_approved_is_invalid() {
        let result = WorkflowValidator::validate_transition(LicenseStatus::Draft, LicenseStatus::Approved);
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::InvalidTransition { from, to } => {
                assert_eq!(from, LicenseStatus::Draft);
                assert_eq!(to, LicenseStatus::Approved);
            }
            _ => panic!("Expected InvalidTransition"),
        }
    }

    #[test]
    fn draft_to_rejected_is_invalid() {
        assert!(WorkflowValidator::validate_transition(LicenseStatus::Draft, LicenseStatus::Rejected).is_err());
    }

    #[test]
    fn approved_to_anything_is_invalid() {
        assert!(WorkflowValidator::validate_transition(LicenseStatus::Approved, LicenseStatus::Draft).is_err());
        assert!(WorkflowValidator::validate_transition(LicenseStatus::Approved, LicenseStatus::Negotiating).is_err());
        assert!(WorkflowValidator::validate_transition(LicenseStatus::Approved, LicenseStatus::Rejected).is_err());
    }

    #[test]
    fn rejected_to_approved_is_invalid() {
        assert!(WorkflowValidator::validate_transition(LicenseStatus::Rejected, LicenseStatus::Approved).is_err());
    }

    #[test]
    fn rejected_to_draft_is_invalid() {
        assert!(WorkflowValidator::validate_transition(LicenseStatus::Rejected, LicenseStatus::Draft).is_err());
    }

    #[test]
    fn full_happy_path() {
        WorkflowValidator::validate_transition(LicenseStatus::Draft, LicenseStatus::Negotiating).unwrap();
        WorkflowValidator::validate_transition(LicenseStatus::Negotiating, LicenseStatus::Approved).unwrap();
    }

    #[test]
    fn rejection_then_renegotiation_path() {
        WorkflowValidator::validate_transition(LicenseStatus::Draft, LicenseStatus::Negotiating).unwrap();
        WorkflowValidator::validate_transition(LicenseStatus::Negotiating, LicenseStatus::Rejected).unwrap();
        WorkflowValidator::validate_transition(LicenseStatus::Rejected, LicenseStatus::Negotiating).unwrap();
        WorkflowValidator::validate_transition(LicenseStatus::Negotiating, LicenseStatus::Approved).unwrap();
    }
}