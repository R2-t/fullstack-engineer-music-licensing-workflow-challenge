use crate::domain::{LicenseAuditEntry, LicenseStatus};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Pagination {
    pub page: i32,
    pub limit: i32,
    pub total: i32,
    pub total_pages: i32,
}

#[derive(Serialize)]
pub struct MovieSummary {
    pub id: i32,
    pub title: String,
    pub release_date: Option<chrono::NaiveDate>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct MovieFull {
    pub id: i32,
    pub title: String,
    pub release_date: Option<chrono::NaiveDate>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub scene_count: i32,
}

#[derive(Deserialize)]
pub struct CreateMovieRequest {
    pub title: String,
    #[serde(default)]
    pub release_date: Option<chrono::NaiveDate>,
}

#[derive(Serialize)]
pub struct MovieListResponse {
    pub items: Vec<MovieSummary>,
    pub pagination: Pagination,
}

#[derive(Serialize)]
pub struct SceneSummary {
    pub id: i32,
    pub movie_id: i32,
    pub scene_number: i16,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct SceneFull {
    pub id: i32,
    pub movie_id: i32,
    pub scene_number: i16,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub track_count: i32,
}

#[derive(Deserialize)]
pub struct CreateSceneRequest {
    pub scene_number: i16,
}

#[derive(Serialize)]
pub struct SceneListResponse {
    pub items: Vec<SceneSummary>,
    pub pagination: Pagination,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SongDto {
    pub title: String,
    pub artist: Option<String>,
    #[serde(rename = "labelName")]
    pub label_name: Option<String>,
    #[serde(rename = "durationSecStart")]
    pub duration_sec_start: i32,
    #[serde(rename = "durationSecEnd")]
    pub duration_sec_end: i32,
}

#[derive(Serialize)]
pub struct TrackSummary {
    pub id: i32,
    pub scene_id: i32,
    pub track_order: i16,
    pub name: String,
    pub song: SongDto,
    pub license_status: Option<LicenseStatus>,
}

#[derive(Serialize)]
pub struct TrackFull {
    pub id: i32,
    pub scene_id: i32,
    pub track_order: i16,
    pub name: String,
    pub song: SongDto,
    pub license_status: Option<LicenseStatus>,
    pub license: Option<LicenseSummary>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct CreateTrackRequest {
    pub track_order: Option<i16>,
    pub name: Option<String>,
    pub song: SongDto,
}

#[derive(Deserialize)]
pub struct PatchTrackRequest {
    pub track_order: Option<i16>,
    pub name: Option<String>,
    pub song: Option<SongDto>,
}

#[derive(Serialize)]
pub struct TrackListResponse {
    pub items: Vec<TrackSummary>,
    pub pagination: Pagination,
}

#[derive(Serialize)]
pub struct LicenseSummary {
    pub id: i32,
    pub track_id: i32,
    pub label_name: Option<String>,
    pub artist_name: Option<String>,
    pub status: LicenseStatus,
    pub last_updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct LicenseFull {
    pub id: i32,
    pub track_id: i32,
    pub label_name: Option<String>,
    pub artist_name: Option<String>,
    pub status: LicenseStatus,
    pub negotiation_notes: serde_json::Value,
    pub last_updated_at: chrono::DateTime<chrono::Utc>,
    pub audit_log: Vec<LicenseAuditEntry>,
}

#[derive(Deserialize)]
pub struct CreateLicenseRequest {
    pub label_name: Option<String>,
    pub artist_name: Option<String>,
    pub negotiation_notes: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct LicenseStatusTransitionRequest {
    pub target_status: LicenseStatus,
    pub notes: Option<serde_json::Value>,
}
