use axum::{
    routing::{get, patch},
    Router,
    middleware,
};
use std::sync::Arc;
use tower_http::trace::{TraceLayer, DefaultMakeSpan};
use crate::adapters::http::handlers::{
    movie_handler, scene_handler, track_handler, license_handler, sse_handler
};
use crate::adapters::http::auth::auth_middleware;
use crate::adapters::db::PostgresAdapter;
use crate::adapters::realtime::BroadcastEventPublisher;
use crate::ports::*;
use sqlx::PgPool;

pub struct AppState {
    pub movie_repo: Arc<dyn MovieRepository>,
    pub scene_repo: Arc<dyn SceneRepository>,
    pub track_repo: Arc<dyn TrackRepository>,
    pub license_repo: Arc<dyn LicenseRepository>,
    pub audit_repo: Arc<dyn AuditRepository>,
    pub event_publisher: Arc<BroadcastEventPublisher>,
    pub jwt_secret: String,
}

pub fn create_router(pool: PgPool, jwt_secret: String) -> Router {
    let adapter = PostgresAdapter::new(pool);
    let publisher = Arc::new(BroadcastEventPublisher::new());

    let state = Arc::new(AppState {
        movie_repo: adapter.movie_repo.clone(),
        scene_repo: adapter.scene_repo.clone(),
        track_repo: adapter.track_repo.clone(),
        license_repo: adapter.license_repo.clone(),
        audit_repo: adapter.audit_repo.clone(),
        event_publisher: publisher.clone(),
        jwt_secret: jwt_secret.clone(),
    });

    let api_routes = Router::new()
        .route("/movies", get(movie_handler::list_movies).post(movie_handler::create_movie))
        .route("/movies/{id}", get(movie_handler::get_movie))
        .route("/movies/{id}/scenes", get(scene_handler::list_scenes).post(scene_handler::create_scene))
        .route("/movies/{movie_id}/scenes/{scene_id}", get(scene_handler::get_scene))
        .route("/movies/{movie_id}/scenes/{scene_id}/tracks", get(track_handler::list_tracks).post(track_handler::create_track))
        .route("/movies/{movie_id}/scenes/{scene_id}/tracks/{track_id}", get(track_handler::get_track).patch(track_handler::update_track).delete(track_handler::delete_track))
        .route("/movies/{movie_id}/scenes/{scene_id}/tracks/{track_id}/licenses", get(license_handler::get_license).post(license_handler::initiate_license))
        .route("/movies/{movie_id}/scenes/{scene_id}/tracks/{track_id}/licenses/status-transition", patch(license_handler::transition_status))
        .route("/movies/{movie_id}/scenes/{scene_id}/tracks/{track_id}/licenses/events", get(sse_handler::stream_events))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    Router::new()
        .merge(api_routes)
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().include_headers(true)))
        .with_state(state)
}