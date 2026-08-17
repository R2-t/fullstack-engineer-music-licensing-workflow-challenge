use std::sync::Arc;
use crate::domain::*;
use crate::ports::SceneRepository;

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