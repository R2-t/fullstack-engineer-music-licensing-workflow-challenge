use std::sync::Arc;
use crate::domain::*;
use crate::ports::TrackRepository;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::MockTrackRepository;
    use mockall::predicate::*;
    use chrono::Utc;

    fn make_song() -> Song {
        Song {
            title: "Test Song".to_string(),
            artist: Some("Artist".to_string()),
            label_name: None,
            duration_sec_start: 0,
            duration_sec_end: 180,
        }
    }

    fn make_track(id: i32, scene_id: i32) -> Track {
        Track {
            id,
            scene_id,
            track_order: 1,
            name: "Track 1".to_string(),
            song: make_song(),
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn create_track_with_valid_song_succeeds() {
        let mut repo = MockTrackRepository::new();
        repo.expect_create()
            .with(eq(1), eq(1), eq("Track 1".to_string()), always())
            .returning(|scene_id, order, name, song| Ok(Track {
                id: 1, scene_id, track_order: order, name, song, created_at: Utc::now(),
            }));

        let service = TrackService::new(Arc::new(repo));
        let result = service.create(1, 1, "Track 1".to_string(), make_song()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn create_track_with_invalid_duration_fails() {
        let repo = MockTrackRepository::new();
        let service = TrackService::new(Arc::new(repo));

        let mut bad_song = make_song();
        bad_song.duration_sec_end = 50;
        bad_song.duration_sec_start = 100;

        let result = service.create(1, 1, "Track 1".to_string(), bad_song).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DomainError::ValidationError(msg) => assert!(msg.contains("duration_sec_end")),
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn create_track_with_equal_start_end_fails() {
        let repo = MockTrackRepository::new();
        let service = TrackService::new(Arc::new(repo));

        let mut bad_song = make_song();
        bad_song.duration_sec_start = 100;
        bad_song.duration_sec_end = 100;

        let result = service.create(1, 1, "Track 1".to_string(), bad_song).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn update_track_with_valid_song_succeeds() {
        let mut repo = MockTrackRepository::new();
        repo.expect_update()
            .returning(|id, _, _, _| Ok(make_track(id, 1)));

        let service = TrackService::new(Arc::new(repo));
        let result = service.update(1, Some("New Name".to_string()), None, Some(make_song())).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn update_track_with_invalid_duration_fails() {
        let repo = MockTrackRepository::new();
        let service = TrackService::new(Arc::new(repo));

        let mut bad_song = make_song();
        bad_song.duration_sec_end = 10;
        bad_song.duration_sec_start = 100;

        let result = service.update(1, None, None, Some(bad_song)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn update_track_without_song_passes_validation() {
        let mut repo = MockTrackRepository::new();
        repo.expect_update()
            .returning(|id, _, _, _| Ok(make_track(id, 1)));

        let service = TrackService::new(Arc::new(repo));
        let result = service.update(1, Some("Updated".to_string()), None, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn find_by_id_delegates_to_repo() {
        let mut repo = MockTrackRepository::new();
        repo.expect_find_by_id()
            .with(eq(1))
            .returning(|_| Ok(make_track(1, 1)));

        let service = TrackService::new(Arc::new(repo));
        let result = service.find_by_id(1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn list_by_scene_delegates_to_repo() {
        let mut repo = MockTrackRepository::new();
        repo.expect_list_by_scene()
            .with(eq(1), eq(1), eq(20))
            .returning(|_, _, _| Ok(vec![make_track(1, 1)]));

        let service = TrackService::new(Arc::new(repo));
        let result = service.list_by_scene(1, 1, 20).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn delete_delegates_to_repo() {
        let mut repo = MockTrackRepository::new();
        repo.expect_delete()
            .with(eq(1))
            .returning(|_| Ok(()));

        let service = TrackService::new(Arc::new(repo));
        let result = service.delete(1).await;
        assert!(result.is_ok());
    }
}