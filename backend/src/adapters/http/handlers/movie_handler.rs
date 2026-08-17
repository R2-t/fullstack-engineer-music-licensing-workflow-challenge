use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::adapters::http::dto::*;
use crate::adapters::http::router::AppState;
use crate::error::AppError;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PaginationParams {
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

pub async fn list_movies(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<MovieListResponse>, AppError> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);
    
    let movies = state.movie_repo.list(page, limit).await?;
    
    Ok(Json(MovieListResponse {
        items: movies.into_iter().map(|m| MovieSummary {
            id: m.id,
            title: m.title,
            release_date: m.release_date,
            created_at: m.created_at,
        }).collect(),
        pagination: Pagination {
            page,
            limit,
            total: 0,
            total_pages: 0,
        },
    }))
}

pub async fn create_movie(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateMovieRequest>,
) -> Result<(StatusCode, Json<MovieSummary>), AppError> {
    let movie = state.movie_repo.create(req.title, req.release_date).await?;
    Ok((StatusCode::CREATED, Json(MovieSummary {
        id: movie.id,
        title: movie.title,
        release_date: movie.release_date,
        created_at: movie.created_at,
    })))
}

pub async fn get_movie(
    Path(id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<MovieFull>, AppError> {
    let movie = state.movie_repo.find_by_id(id).await?;
    Ok(Json(MovieFull {
        id: movie.id,
        title: movie.title,
        release_date: movie.release_date,
        created_at: movie.created_at,
        scene_count: 0,
    }))
}