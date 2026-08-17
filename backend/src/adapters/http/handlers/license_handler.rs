use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::adapters::http::dto::*;
use crate::adapters::http::router::AppState;
use crate::domain::workflow::WorkflowValidator;
use crate::error::AppError;
use crate::ports::EventPublisher;

pub async fn get_license(
    Path((_movie_id, _scene_id, track_id)): Path<(i32, i32, i32)>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<LicenseFull>, AppError> {
    let license = state.license_repo.find_by_track(track_id).await?
        .ok_or_else(|| AppError::NotFound(format!("License for track {}", track_id)))?;
    
    let audit_log = state.audit_repo.list_by_license(license.id).await.unwrap_or_default();

    Ok(Json(LicenseFull {
        id: license.id,
        track_id: license.track_id,
        label_name: license.label_name,
        artist_name: license.artist_name,
        status: license.status,
        negotiation_notes: license.negotiation_notes,
        last_updated_at: license.last_updated_at,
        audit_log,
    }))
}

pub async fn initiate_license(
    Path((_movie_id, _scene_id, track_id)): Path<(i32, i32, i32)>,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateLicenseRequest>,
) -> Result<(StatusCode, Json<LicenseFull>), AppError> {
    if state.license_repo.find_by_track(track_id).await?.is_some() {
        return Err(AppError::Conflict("A license already exists for this track".to_string()));
    }
    
    let notes = req.negotiation_notes.unwrap_or_else(|| serde_json::json!({}));
    let license = state.license_repo.create(track_id, req.label_name, req.artist_name, notes).await?;
    
    state.audit_repo.log_transition(
        track_id, license.id, "system".to_string(), None, crate::domain::LicenseStatus::Draft
    ).await.ok();

    let audit_log = state.audit_repo.list_by_license(license.id).await.unwrap_or_default();

    Ok((StatusCode::CREATED, Json(LicenseFull {
        id: license.id,
        track_id: license.track_id,
        label_name: license.label_name,
        artist_name: license.artist_name,
        status: license.status,
        negotiation_notes: license.negotiation_notes,
        last_updated_at: license.last_updated_at,
        audit_log,
    })))
}

pub async fn transition_status(
    Path((_movie_id, _scene_id, track_id)): Path<(i32, i32, i32)>,
    State(state): State<Arc<AppState>>,
    Json(req): Json<LicenseStatusTransitionRequest>,
) -> Result<Json<LicenseFull>, AppError> {
    let license = state.license_repo.find_by_track(track_id).await?
        .ok_or_else(|| AppError::NotFound(format!("License for track {}", track_id)))?;
    
    WorkflowValidator::validate_transition(license.status, req.target_status)
        .map_err(|e| AppError::from(e))?;

    let notes = req.notes.unwrap_or_else(|| serde_json::json!({}));
    let updated = state.license_repo.update_status(license.id, req.target_status, notes).await?;
    
    let user = "authenticated_user@example.com".to_string();
    
    state.audit_repo.log_transition(
        track_id, license.id, user.clone(), Some(license.status), req.target_status
    ).await.ok();

    state.event_publisher.publish_status_changed(crate::ports::LicenseEvent {
        track_id,
        license_id: license.id,
        previous_status: Some(license.status),
        status: updated.status,
        changed_by: user,
        timestamp: chrono::Utc::now(),
    }).await;

    let audit_log = state.audit_repo.list_by_license(license.id).await.unwrap_or_default();

    Ok(Json(LicenseFull {
        id: updated.id,
        track_id: updated.track_id,
        label_name: updated.label_name,
        artist_name: updated.artist_name,
        status: updated.status,
        negotiation_notes: updated.negotiation_notes,
        last_updated_at: updated.last_updated_at,
        audit_log,
    }))
}