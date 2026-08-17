pub mod services;

use std::sync::Arc;
use crate::domain::*;
use crate::ports::*;
use async_trait::async_trait;

pub struct MovieService<R: MovieRepository> {
    repo: Arc<R>,
}

impl<R: MovieRepository> MovieService<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    pub async fn list(&self, page: i32, limit: i32) -> Result<Vec<Movie>, DomainError> {
        self.repo.list(page, limit).await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Movie, DomainError> {
        self.repo.find_by_id(id).await
    }

    pub async fn create(&self, title: String, release_date: Option<chrono::NaiveDate>) -> Result<Movie, DomainError> {
        self.repo.create(title, release_date).await
    }
}

pub struct SceneService<R: SceneRepository> {
    repo: Arc<R>,
}

impl<R: SceneRepository> SceneService<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    pub async fn list_by_movie(&self, movie_id: i32, page: i32, limit: i32) -> Result<Vec<Scene>, DomainError> {
        self.repo.list_by_movie(movie_id, page, limit).await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Scene, DomainError> {
        self.repo.find_by_id(id).await
    }

    pub async fn create(&self, movie_id: i32, scene_number: i16) -> Result<Scene, DomainError> {
        self.repo.create(movie_id, scene_number).await
    }
}

pub struct TrackService<R: TrackRepository> {
    repo: Arc<R>,
}

impl<R: TrackRepository> TrackService<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    pub async fn list_by_scene(&self, scene_id: i32, page: i32, limit: i32) -> Result<Vec<Track>, DomainError> {
        self.repo.list_by_scene(scene_id, page, limit).await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Track, DomainError> {
        self.repo.find_by_id(id).await
    }

    pub async fn create(&self, scene_id: i32, order: i16, name: String, song: Song) -> Result<Track, DomainError> {
        if song.duration_sec_end <= song.duration_sec_start {
            return Err(DomainError::ValidationError(
                "duration_sec_end must be greater than duration_sec_start".to_string(),
            ));
        }
        self.repo.create(scene_id, order, name, song).await
    }

    pub async fn update(&self, id: i32, name: Option<String>, order: Option<i16>, song: Option<Song>) -> Result<Track, DomainError> {
        if let Some(ref s) = song {
            if s.duration_sec_end <= s.duration_sec_start {
                return Err(DomainError::ValidationError(
                    "duration_sec_end must be greater than duration_sec_start".to_string(),
                ));
            }
        }
        self.repo.update(id, name, order, song).await
    }

    pub async fn delete(&self, id: i32) -> Result<(), DomainError> {
        self.repo.delete(id).await
    }
}