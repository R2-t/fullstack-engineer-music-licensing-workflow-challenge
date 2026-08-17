use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::adapters::http::dto::*;
use crate::adapters::http::router::AppState;
use crate::adapters::http::handlers::movie_handler::PaginationParams;
use crate::error::AppError;

pub async fn list_scenes(
    Path(movie_id): Path<i32>,
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<SceneListResponse>, AppError> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);
    
    let scenes = state.scene_repo.list_by_movie(movie_id, page, limit).await?;
    
    Ok(Json(SceneListResponse {
        items: scenes.into_iter().map(|s| SceneSummary {
            id: s.id,
            movie_id: s.movie_id,
            scene_number: s.scene_number,
            created_at: s.created_at,
        }).collect(),
        pagination: Pagination {
            page,
            limit,
            total: 0,
            total_pages: 0,
        },
    }))
}

pub async fn create_scene(
    Path(movie_id): Path<i32>,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateSceneRequest>,
) -> Result<(StatusCode, Json<SceneFull>), AppError> {
    let scene = state.scene_repo.create(movie_id, req.scene_number).await?;
    Ok((StatusCode::CREATED, Json(SceneFull {
        id: scene.id,
        movie_id: scene.movie_id,
        scene_number: scene.scene_number,
        created_at: scene.created_at,
        track_count: 0,
    })))
}

pub async fn get_scene(
    Path((_movie_id, scene_id)): Path<(i32, i32)>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<SceneFull>, AppError> {
    let scene = state.scene_repo.find_by_id(scene_id).await?;
    Ok(Json(SceneFull {
        id: scene.id,
        movie_id: scene.movie_id,
        scene_number: scene.scene_number,
        created_at: scene.created_at,
        track_count: 0,
    }))
}