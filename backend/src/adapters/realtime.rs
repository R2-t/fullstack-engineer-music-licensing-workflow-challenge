use crate::domain::DomainError;
use crate::ports::{
    EventPublisher, EventSubscriber, EventSubscriberFactory, LicenseEvent, StreamMessage,
};
use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::streams::{
    StreamAutoClaimOptions, StreamAutoClaimReply, StreamRangeReply, StreamReadOptions,
    StreamReadReply,
};
use redis::{AsyncCommands, Client, RedisError, Value};
use std::collections::HashMap;
use tracing::{error, instrument};

const STREAM_KEY: &str = "licenses:events";
const GROUP_NAME: &str = "events-grp";

fn map_redis_err(e: RedisError) -> DomainError {
    error!("Redis error: {}", e);
    DomainError::Internal
}

fn parse_stream_id(id: &str, map: &HashMap<String, Value>) -> Result<StreamMessage, DomainError> {
    let json: String = match map.get("event") {
        Some(Value::BulkString(ref bytes)) => String::from_utf8(bytes.clone()).map_err(|e| {
            error!("Invalid UTF-8 in stream entry {}: {}", id, e);
            DomainError::Internal
        })?,
        _ => {
            error!("Missing 'event' field in stream entry {}", id);
            return Err(DomainError::Internal);
        }
    };

    let mut event: LicenseEvent = serde_json::from_str(&json).map_err(|e| {
        error!(
            "Failed to deserialize LicenseEvent from stream entry {}: {}",
            id, e
        );
        DomainError::Internal
    })?;

    event.stream_id = id.to_string();
    Ok(StreamMessage {
        id: id.to_string(),
        event,
    })
}

fn stream_ids_to_messages(
    reply: impl IntoIterator<Item = redis::streams::StreamId>,
) -> Result<Vec<StreamMessage>, DomainError> {
    reply
        .into_iter()
        .map(|sid| parse_stream_id(&sid.id, &sid.map))
        .collect()
}

pub struct RedisEventHub {
    conn: ConnectionManager,
}

impl RedisEventHub {
    pub async fn new(redis_url: &str) -> Result<Self, DomainError> {
        let client = Client::open(redis_url).map_err(map_redis_err)?;
        let conn = ConnectionManager::new(client)
            .await
            .map_err(map_redis_err)?;
        let hub = Self { conn };
        hub.ensure_group().await?;
        Ok(hub)
    }

    async fn ensure_group(&self) -> Result<(), DomainError> {
        let mut conn = self.conn.clone();
        let result: Result<(), RedisError> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(STREAM_KEY)
            .arg(GROUP_NAME)
            .arg("$")
            .arg("MKSTREAM")
            .query_async(&mut conn)
            .await;
        if let Err(e) = result {
            let s = e.to_string();
            if !s.contains("BUSYGROUP") {
                return Err(map_redis_err(e));
            }
        }
        Ok(())
    }
}

impl Clone for RedisEventHub {
    fn clone(&self) -> Self {
        Self {
            conn: self.conn.clone(),
        }
    }
}

#[async_trait]
impl EventPublisher for RedisEventHub {
    #[instrument(skip(self, event), name = "redis.publish", fields(track_id = %event.track_id))]
    async fn publish_status_changed(&self, mut event: LicenseEvent) -> Result<String, DomainError> {
        let json = serde_json::to_string(&event).map_err(|e| {
            error!("Failed to serialize LicenseEvent: {}", e);
            DomainError::Internal
        })?;
        let mut conn = self.conn.clone();
        let id: String = conn
            .xadd(STREAM_KEY, "*", &[("event", json)])
            .await
            .map_err(map_redis_err)?;
        event.stream_id = id.clone();
        Ok(id)
    }
}

impl EventSubscriberFactory for RedisEventHub {
    fn create_subscriber(
        &self,
        consumer_name: String,
    ) -> Result<Box<dyn EventSubscriber>, DomainError> {
        Ok(Box::new(RedisEventSubscriber::new(
            self.conn.clone(),
            consumer_name,
        )))
    }
}

pub struct RedisEventSubscriber {
    conn: ConnectionManager,
    consumer_name: String,
}

impl RedisEventSubscriber {
    pub fn new(conn: ConnectionManager, consumer_name: String) -> Self {
        Self {
            conn,
            consumer_name,
        }
    }
}

#[async_trait]
impl EventSubscriber for RedisEventSubscriber {
    fn consumer_name(&self) -> String {
        self.consumer_name.clone()
    }

    #[instrument(skip(self), name = "redis.replay")]
    async fn replay(
        &self,
        last_event_id: Option<String>,
        limit: i64,
    ) -> Result<Vec<StreamMessage>, DomainError> {
        let start = last_event_id
            .map(|id| format!("({}", id))
            .unwrap_or_else(|| "-".to_string());
        let mut conn = self.conn.clone();
        let reply: StreamRangeReply = conn
            .xrange_count(STREAM_KEY, &start, "+", limit as usize)
            .await
            .map_err(map_redis_err)?;
        stream_ids_to_messages(reply.ids)
    }

    #[instrument(skip(self), name = "redis.next_event")]
    async fn next_event(&self, block_ms: u64) -> Result<Option<StreamMessage>, DomainError> {
        let opts = StreamReadOptions::default()
            .group(GROUP_NAME, &self.consumer_name)
            .count(1)
            .block(block_ms as usize);
        let mut conn = self.conn.clone();
        let reply: StreamReadReply = conn
            .xread_options(&[STREAM_KEY], &[">"], &opts)
            .await
            .map_err(map_redis_err)?;
        let messages: Vec<StreamMessage> = reply
            .keys
            .into_iter()
            .flat_map(|k| k.ids)
            .map(|sid| parse_stream_id(&sid.id, &sid.map))
            .collect::<Result<_, _>>()?;
        Ok(messages.into_iter().next())
    }

    #[instrument(skip(self), name = "redis.ack")]
    async fn ack(&self, id: &str) -> Result<(), DomainError> {
        let mut conn = self.conn.clone();
        let _: i64 = conn
            .xack(STREAM_KEY, GROUP_NAME, &[id])
            .await
            .map_err(map_redis_err)?;
        Ok(())
    }

    #[instrument(skip(self), name = "redis.claim")]
    async fn claim(&self, idle_ms: u64) -> Result<Vec<StreamMessage>, DomainError> {
        let opts = StreamAutoClaimOptions::default().count(100);
        let mut conn = self.conn.clone();
        let reply: StreamAutoClaimReply = conn
            .xautoclaim_options(
                STREAM_KEY,
                GROUP_NAME,
                &self.consumer_name,
                idle_ms,
                "0",
                opts,
            )
            .await
            .map_err(map_redis_err)?;
        stream_ids_to_messages(reply.claimed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::LicenseStatus;

    fn make_event(track_id: i32, license_id: i32, status: LicenseStatus) -> LicenseEvent {
        LicenseEvent {
            stream_id: String::new(),
            track_id,
            license_id,
            previous_status: Some(LicenseStatus::Draft),
            status,
            changed_by: "test@example.com".to_string(),
            timestamp: chrono::Utc::now(),
        }
    }

    #[test]
    fn license_event_serializes_with_stream_id() {
        let event = make_event(1, 10, LicenseStatus::Negotiating);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("stream_id"));
        let deserialized: LicenseEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.track_id, 1);
        assert_eq!(deserialized.stream_id, "");
    }

    #[test]
    fn stream_message_serializes() {
        let msg = StreamMessage {
            id: "1710000000123-5".to_string(),
            event: make_event(1, 10, LicenseStatus::Approved),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("1710000000123-5"));
        assert!(json.contains("APPROVED"));
    }
}
