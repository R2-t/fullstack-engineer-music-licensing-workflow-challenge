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