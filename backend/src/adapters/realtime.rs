use tokio::sync::broadcast;
use crate::ports::EventPublisher;
use crate::ports::LicenseEvent;

pub struct BroadcastEventPublisher {
    tx: broadcast::Sender<LicenseEvent>,
}

impl BroadcastEventPublisher {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<LicenseEvent> {
        self.tx.subscribe()
    }
}

#[async_trait::async_trait]
impl EventPublisher for BroadcastEventPublisher {
    async fn publish_status_changed(&self, event: LicenseEvent) {
        let _ = self.tx.send(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::LicenseStatus;

    fn make_event(track_id: i32, license_id: i32, status: LicenseStatus) -> LicenseEvent {
        LicenseEvent {
            track_id,
            license_id,
            previous_status: Some(LicenseStatus::Draft),
            status,
            changed_by: "test@example.com".to_string(),
            timestamp: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn publish_and_receive_event() {
        let publisher = BroadcastEventPublisher::new();
        let mut rx = publisher.subscribe();

        let event = make_event(1, 10, LicenseStatus::Negotiating);
        publisher.publish_status_changed(event.clone()).await;

        let received = rx.try_recv().unwrap();
        assert_eq!(received.track_id, 1);
        assert_eq!(received.license_id, 10);
        assert_eq!(received.status, LicenseStatus::Negotiating);
    }

    #[tokio::test]
    async fn multiple_subscribers_receive_event() {
        let publisher = BroadcastEventPublisher::new();
        let mut rx1 = publisher.subscribe();
        let mut rx2 = publisher.subscribe();

        let event = make_event(5, 20, LicenseStatus::Approved);
        publisher.publish_status_changed(event).await;

        let r1 = rx1.try_recv().unwrap();
        let r2 = rx2.try_recv().unwrap();
        assert_eq!(r1.track_id, 5);
        assert_eq!(r2.track_id, 5);
    }

    #[tokio::test]
    async fn subscribe_after_publish_misses_event() {
        let publisher = BroadcastEventPublisher::new();
        let event = make_event(1, 1, LicenseStatus::Draft);
        publisher.publish_status_changed(event).await;

        let mut rx = publisher.subscribe();
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn publish_multiple_events() {
        let publisher = BroadcastEventPublisher::new();
        let mut rx = publisher.subscribe();

        for i in 1..=5 {
            publisher.publish_status_changed(make_event(i, i * 10, LicenseStatus::Draft)).await;
        }

        for i in 1..=5 {
            let event = rx.try_recv().unwrap();
            assert_eq!(event.track_id, i);
        }
    }
}