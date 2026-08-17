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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::MockSceneRepository;
    use mockall::predicate::*;
    use chrono::Utc;

    fn make_scene(id: i32, movie_id: i32) -> Scene {
        Scene {
            id,
            movie_id,
            scene_number: id as i16,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn list_by_movie_delegates_to_repo() {
        let mut repo = MockSceneRepository::new();
        repo.expect_list_by_movie()
            .with(eq(1), eq(1), eq(20))
            .returning(|_, _, _| Ok(vec![make_scene(1, 1)]));

        let service = SceneService::new(Arc::new(repo));
        let result = service.list_by_movie(1, 1, 20).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn find_by_id_found() {
        let mut repo = MockSceneRepository::new();
        repo.expect_find_by_id()
            .with(eq(1))
            .returning(|_| Ok(make_scene(1, 1)));

        let service = SceneService::new(Arc::new(repo));
        let result = service.find_by_id(1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn create_scene_delegates_to_repo() {
        let mut repo = MockSceneRepository::new();
        repo.expect_create()
            .with(eq(1), eq(5))
            .returning(|movie_id, scene_number| Ok(Scene {
                id: 1,
                movie_id,
                scene_number,
                created_at: Utc::now(),
            }));

        let service = SceneService::new(Arc::new(repo));
        let result = service.create(1, 5).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().scene_number, 5);
    }
}