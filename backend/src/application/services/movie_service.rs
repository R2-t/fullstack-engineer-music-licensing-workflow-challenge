use crate::domain::*;
use crate::ports::MovieRepository;
use std::sync::Arc;

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

    pub async fn create(
        &self,
        title: String,
        release_date: Option<chrono::NaiveDate>,
    ) -> Result<Movie, DomainError> {
        self.repo.create(title, release_date).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::MockMovieRepository;
    use chrono::Utc;
    use mockall::predicate::*;

    fn make_movie(id: i32) -> Movie {
        Movie {
            id,
            title: format!("Movie {}", id),
            release_date: None,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn list_movies_delegates_to_repo() {
        let mut repo = MockMovieRepository::new();
        repo.expect_list()
            .with(eq(1), eq(20))
            .returning(|_, _| Ok(vec![make_movie(1), make_movie(2)]));

        let service = MovieService::new(Arc::new(repo));
        let result = service.list(1, 20).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn find_by_id_found() {
        let mut repo = MockMovieRepository::new();
        repo.expect_find_by_id()
            .with(eq(1))
            .returning(|_| Ok(make_movie(1)));

        let service = MovieService::new(Arc::new(repo));
        let result = service.find_by_id(1).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, 1);
    }

    #[tokio::test]
    async fn find_by_id_not_found() {
        let mut repo = MockMovieRepository::new();
        repo.expect_find_by_id()
            .with(eq(999))
            .returning(|_| Err(DomainError::NotFound("Movie 999".to_string())));

        let service = MovieService::new(Arc::new(repo));
        let result = service.find_by_id(999).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_movie_delegates_to_repo() {
        let mut repo = MockMovieRepository::new();
        repo.expect_create()
            .with(eq("New Movie".to_string()), eq(None))
            .returning(|title, date| {
                Ok(Movie {
                    id: 1,
                    title,
                    release_date: date,
                    created_at: Utc::now(),
                })
            });

        let service = MovieService::new(Arc::new(repo));
        let result = service.create("New Movie".to_string(), None).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().title, "New Movie");
    }
}
