use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Error)]
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
