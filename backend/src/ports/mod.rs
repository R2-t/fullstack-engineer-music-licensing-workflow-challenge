use async_trait::async_trait;
use crate::domain::*;
use serde::{Serialize, Deserialize};

#[async_trait]
pub trait MovieRepository: Send + Sync {
    async fn list(&self, page: i32, limit: i32) -> Result<Vec<Movie>, DomainError>;
    async fn find_by_id(&self, id: i32) -> Result<Movie, DomainError>;
    async fn create(&self, title: String, release_date: Option<chrono::NaiveDate>) -> Result<Movie, DomainError>;
}

#[async_trait]
pub trait SceneRepository: Send + Sync {
    async fn list_by_movie(&self, movie_id: i32, page: i32, limit: i32) -> Result<Vec<Scene>, DomainError>;
    async fn find_by_id(&self, id: i32) -> Result<Scene, DomainError>;
    async fn create(&self, movie_id: i32, scene_number: i16) -> Result<Scene, DomainError>;
}

#[async_trait]
pub trait TrackRepository: Send + Sync {
    async fn list_by_scene(&self, scene_id: i32, page: i32, limit: i32) -> Result<Vec<Track>, DomainError>;
    async fn find_by_id(&self, id: i32) -> Result<Track, DomainError>;
    async fn create(&self, scene_id: i32, order: i16, name: String, song: Song) -> Result<Track, DomainError>;
    async fn update(&self, id: i32, name: Option<String>, order: Option<i16>, song: Option<Song>) -> Result<Track, DomainError>;
    async fn delete(&self, id: i32) -> Result<(), DomainError>;
}

#[async_trait]
pub trait LicenseRepository: Send + Sync {
    async fn find_by_track(&self, track_id: i32) -> Result<Option<License>, DomainError>;
    async fn find_by_id(&self, id: i32) -> Result<License, DomainError>;
    async fn create(&self, track_id: i32, label: Option<String>, artist: Option<String>, notes: serde_json::Value) -> Result<License, DomainError>;
    async fn update_status(&self, id: i32, status: LicenseStatus, notes: serde_json::Value) -> Result<License, DomainError>;
}

#[async_trait]
pub trait AuditRepository: Send + Sync {
    async fn log_transition(&self, track_id: i32, license_id: i32, user: String, old: Option<LicenseStatus>, new: LicenseStatus) -> Result<LicenseAuditEntry, DomainError>;
    async fn list_by_license(&self, license_id: i32) -> Result<Vec<LicenseAuditEntry>, DomainError>;
    async fn find_after_id(&self, license_id: i32, after_id: i32) -> Result<Vec<LicenseAuditEntry>, DomainError>;
}

#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish_status_changed(&self, event: LicenseEvent);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseEvent {
    pub track_id: i32,
    pub license_id: i32,
    pub previous_status: Option<LicenseStatus>,
    pub status: LicenseStatus,
    pub changed_by: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
