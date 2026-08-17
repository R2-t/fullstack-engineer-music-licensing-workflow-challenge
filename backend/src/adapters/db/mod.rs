pub mod postgres;

use sqlx::PgPool;
use crate::ports::*;
use std::sync::Arc;

pub struct PostgresAdapter {
    pub movie_repo: Arc<dyn MovieRepository>,
    pub scene_repo: Arc<dyn SceneRepository>,
    pub track_repo: Arc<dyn TrackRepository>,
    pub license_repo: Arc<dyn LicenseRepository>,
    pub audit_repo: Arc<dyn AuditRepository>,
}

impl PostgresAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self {
            movie_repo: Arc::new(postgres::movie_repo_impl::PostgresMovieRepository::new(pool.clone())),
            scene_repo: Arc::new(postgres::scene_repo_impl::PostgresSceneRepository::new(pool.clone())),
            track_repo: Arc::new(postgres::track_repo_impl::PostgresTrackRepository::new(pool.clone())),
            license_repo: Arc::new(postgres::license_repo_impl::PostgresLicenseRepository::new(pool.clone())),
            audit_repo: Arc::new(postgres::audit_repo_impl::PostgresAuditRepository::new(pool)),
        }
    }
}