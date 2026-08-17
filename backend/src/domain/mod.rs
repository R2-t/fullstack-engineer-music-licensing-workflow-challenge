pub mod workflow;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use chrono::{DateTime, Utc};

#[derive(Debug, Error, PartialEq)]
pub enum DomainError {
    #[error("Entity not found: {0}")]
    NotFound(String),
    #[error("Invalid transition from {from:?} to {to:?}")]
    InvalidTransition { from: LicenseStatus, to: LicenseStatus },
    #[error("Business rule violation: {0}")]
    ValidationError(String),
    #[error("Internal system error")]
    Internal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LicenseStatus {
    Draft,
    Negotiating,
    Approved,
    Rejected,
}

impl std::fmt::Display for LicenseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "DRAFT"),
            Self::Negotiating => write!(f, "NEGOTIATING"),
            Self::Approved => write!(f, "APPROVED"),
            Self::Rejected => write!(f, "REJECTED"),
        }
    }
}

impl std::str::FromStr for LicenseStatus {
    type Err = DomainError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DRAFT" => Ok(Self::Draft),
            "NEGOTIATING" => Ok(Self::Negotiating),
            "APPROVED" => Ok(Self::Approved),
            "REJECTED" => Ok(Self::Rejected),
            _ => Err(DomainError::ValidationError(format!("Invalid status: {}", s))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Movie {
    pub id: i32,
    pub title: String,
    pub release_date: Option<chrono::NaiveDate>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Scene {
    pub id: i32,
    pub movie_id: i32,
    pub scene_number: i16,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Song {
    pub title: String,
    pub artist: Option<String>,
    pub label_name: Option<String>,
    pub duration_sec_start: i32,
    pub duration_sec_end: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Track {
    pub id: i32,
    pub scene_id: i32,
    pub track_order: i16,
    pub name: String,
    pub song: Song,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct License {
    pub id: i32,
    pub track_id: i32,
    pub label_name: Option<String>,
    pub artist_name: Option<String>,
    pub status: LicenseStatus,
    pub negotiation_notes: serde_json::Value,
    pub last_updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LicenseAuditEntry {
    pub id: i32,
    pub track_id: i32,
    pub license_id: i32,
    pub user_email: String,
    pub old_status: Option<LicenseStatus>,
    pub new_status: LicenseStatus,
    pub changed_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn license_status_display() {
        assert_eq!(LicenseStatus::Draft.to_string(), "DRAFT");
        assert_eq!(LicenseStatus::Negotiating.to_string(), "NEGOTIATING");
        assert_eq!(LicenseStatus::Approved.to_string(), "APPROVED");
        assert_eq!(LicenseStatus::Rejected.to_string(), "REJECTED");
    }

    #[test]
    fn license_status_from_str_valid() {
        assert_eq!("DRAFT".parse::<LicenseStatus>(), Ok(LicenseStatus::Draft));
        assert_eq!("NEGOTIATING".parse::<LicenseStatus>(), Ok(LicenseStatus::Negotiating));
        assert_eq!("APPROVED".parse::<LicenseStatus>(), Ok(LicenseStatus::Approved));
        assert_eq!("REJECTED".parse::<LicenseStatus>(), Ok(LicenseStatus::Rejected));
    }

    #[test]
    fn license_status_from_str_case_insensitive() {
        assert_eq!("draft".parse::<LicenseStatus>(), Ok(LicenseStatus::Draft));
        assert_eq!("Negotiating".parse::<LicenseStatus>(), Ok(LicenseStatus::Negotiating));
        assert_eq!("Approved".parse::<LicenseStatus>(), Ok(LicenseStatus::Approved));
        assert_eq!("REJECTED".parse::<LicenseStatus>(), Ok(LicenseStatus::Rejected));
    }

    #[test]
    fn license_status_from_str_invalid() {
        let result = "INVALID".parse::<LicenseStatus>();
        assert!(result.is_err());
        match result {
            Err(DomainError::ValidationError(msg)) => assert!(msg.contains("INVALID")),
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn domain_error_display() {
        let e = DomainError::NotFound("Movie 1".to_string());
        assert_eq!(e.to_string(), "Entity not found: Movie 1");

        let e = DomainError::ValidationError("bad input".to_string());
        assert_eq!(e.to_string(), "Business rule violation: bad input");

        let e = DomainError::Internal;
        assert_eq!(e.to_string(), "Internal system error");
    }

    #[test]
    fn domain_error_invalid_transition_display() {
        let e = DomainError::InvalidTransition {
            from: LicenseStatus::Draft,
            to: LicenseStatus::Approved,
        };
        assert!(e.to_string().contains("Draft"));
        assert!(e.to_string().contains("Approved"));
    }

    #[test]
    fn license_status_equality() {
        assert_eq!(LicenseStatus::Draft, LicenseStatus::Draft);
        assert_ne!(LicenseStatus::Draft, LicenseStatus::Approved);
    }

    #[test]
    fn song_serialization() {
        let song = Song {
            title: "Bohemian Rhapsody".to_string(),
            artist: Some("Queen".to_string()),
            label_name: Some("EMI".to_string()),
            duration_sec_start: 0,
            duration_sec_end: 354,
        };
        let json = serde_json::to_string(&song).unwrap();
        assert!(json.contains("Bohemian Rhapsody"));
        let deserialized: Song = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.title, song.title);
        assert_eq!(deserialized.artist, song.artist);
    }

    #[test]
    fn license_serialization() {
        let license = License {
            id: 1,
            track_id: 10,
            label_name: Some("Label".to_string()),
            artist_name: None,
            status: LicenseStatus::Negotiating,
            negotiation_notes: serde_json::json!({"note": "test"}),
            last_updated_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&license).unwrap();
        assert!(json.contains("NEGOTIATING"));
        let deserialized: License = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.status, LicenseStatus::Negotiating);
    }
}