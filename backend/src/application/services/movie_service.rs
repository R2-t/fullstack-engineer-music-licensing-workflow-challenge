use std::sync::Arc;
use crate::domain::*;
use crate::ports::MovieRepository;
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