use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::adapters::http::dto::*;
use crate::adapters::http::router::AppState;
use crate::adapters::http::handlers::movie_handler::PaginationParams;
use crate::domain::Song;
use crate::error::AppError;

fn song_dto_to_domain(dto: SongDto) -> Song {
    Song {
        title: dto.title,
        artist: dto.artist,
        label_name: dto.label_name,
        duration_sec_start: dto.duration_sec_start,
        duration_sec_end: dto.duration_sec_end,
    }
}

fn song_domain_to_dto(song: &Song) -> SongDto {
    SongDto {
        title: song.title.clone(),
        artist: song.artist.clone(),
        label_name: song.label_name.clone(),
        duration_sec_start: song.duration_sec_start,
        duration_sec_end: song.duration_sec_end,
    }
}

pub async fn list_tracks(
    Path((_movie_id, scene_id)): Path<(i32, i32)>,
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<TrackListResponse>, AppError> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);
    
    let tracks = state.track_repo.list_by_scene(scene_id, page, limit).await?;
    
    let mut summaries = Vec::new();
    for track in tracks {
        let license_status = state.license_repo.find_by_track(track.id).await.ok().flatten().map(|l| l.status);
        summaries.push(TrackSummary {
            id: track.id,
            scene_id: track.scene_id,
            track_order: track.track_order,
            name: track.name,
            song: song_domain_to_dto(&track.song),
            license_status,
        });
    }

    Ok(Json(TrackListResponse {
        items: summaries,
        pagination: Pagination {
            page,
            limit,
            total: 0,
            total_pages: 0,
        },
    }))
}

pub async fn create_track(
    Path((_movie_id, scene_id)): Path<(i32, i32)>,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateTrackRequest>,
) -> Result<(StatusCode, Json<TrackFull>), AppError> {
    let name = req.name.unwrap_or_else(|| "Unnamed Track".to_string());
    let order = req.track_order.unwrap_or(0);
    
    if req.song.duration_sec_end <= req.song.duration_sec_start {
        return Err(AppError::BadRequest("duration_sec_end must be greater than duration_sec_start".to_string()));
    }
    
    let song = song_dto_to_domain(req.song);
    let track = state.track_repo.create(scene_id, order, name, song).await?;
    
    Ok((StatusCode::CREATED, Json(TrackFull {
        id: track.id,
        scene_id: track.scene_id,
        track_order: track.track_order,
        name: track.name,
        song: song_domain_to_dto(&track.song),
        license_status: None,
        license: None,
        created_at: track.created_at,
    })))
}

pub async fn get_track(
    Path((_movie_id, _scene_id, track_id)): Path<(i32, i32, i32)>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<TrackFull>, AppError> {
    let track = state.track_repo.find_by_id(track_id).await?;
    let license = state.license_repo.find_by_track(track_id).await.ok().flatten();
    
    let license_status = license.as_ref().map(|l| l.status);
    let license_summary = license.map(|l| LicenseSummary {
        id: l.id,
        track_id: l.track_id,
        label_name: l.label_name,
        artist_name: l.artist_name,
        status: l.status,
        last_updated_at: l.last_updated_at,
    });

    Ok(Json(TrackFull {
        id: track.id,
        scene_id: track.scene_id,
        track_order: track.track_order,
        name: track.name,
        song: song_domain_to_dto(&track.song),
        license_status,
        license: license_summary,
        created_at: track.created_at,
    }))
}

pub async fn update_track(
    Path((_movie_id, _scene_id, track_id)): Path<(i32, i32, i32)>,
    State(state): State<Arc<AppState>>,
    Json(req): Json<PatchTrackRequest>,
) -> Result<Json<TrackFull>, AppError> {
    if let Some(ref s) = req.song {
        if s.duration_sec_end <= s.duration_sec_start {
            return Err(AppError::BadRequest("duration_sec_end must be greater than duration_sec_start".to_string()));
        }
    }
    
    let song = req.song.map(song_dto_to_domain);
    let updated = state.track_repo.update(track_id, req.name, req.track_order, song).await?;
    
    Ok(Json(TrackFull {
        id: updated.id,
        scene_id: updated.scene_id,
        track_order: updated.track_order,
        name: updated.name,
        song: song_domain_to_dto(&updated.song),
        license_status: None,
        license: None,
        created_at: updated.created_at,
    }))
}

pub async fn delete_track(
    Path((_movie_id, _scene_id, track_id)): Path<(i32, i32, i32)>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, AppError> {
    state.track_repo.delete(track_id).await?;
    Ok(StatusCode::NO_CONTENT)
}