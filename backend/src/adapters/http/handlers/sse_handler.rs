use crate::adapters::http::router::AppState;
use crate::error::AppError;
#[allow(unused_imports)]
use crate::ports::{EventSubscriber, EventSubscriberFactory};
use axum::extract::{Path, Query, State};
use axum::response::sse::{Event, Sse};
use futures::stream::Stream;
use serde::Deserialize;
use std::convert::Infallible;
use std::sync::Arc;
use tracing::{info, warn, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

const BLOCK_MS: u64 = 5000;
const CLAIM_IDLE_MS: u64 = 60_000;
const REPLAY_LIMIT: i64 = 1000;

#[derive(Debug, Deserialize)]
pub struct SseQuery {
    pub last_event_id: Option<String>,
}

pub async fn stream_events(
    Path((_movie_id, _scene_id, track_id)): Path<(i32, i32, i32)>,
    Query(query): Query<SseQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let span = Span::current();
    span.set_attribute("track.id", track_id as i64);

    let consumer_name = format!("sse-{}", uuid::Uuid::new_v4());

    info!(
        track_id,
        consumer = %consumer_name,
        last_event_id = ?query.last_event_id,
        "SSE connection opened for license events"
    );

    let _last_event_id = query.last_event_id.clone();

    let subscriber = state
        .event_hub
        .create_subscriber(consumer_name.clone())
        .map_err(|e| {
            warn!("Failed to create subscriber: {}", e);
            AppError::Internal(anyhow::anyhow!("Failed to create subscriber"))
        })?;

    let last_for_replay = query.last_event_id.clone();
    let replayed: Vec<crate::ports::StreamMessage> =
        match subscriber.replay(last_for_replay, REPLAY_LIMIT).await {
            Ok(entries) => {
                info!(track_id, count = entries.len(), "Replayed missed events");
                entries
            }
            Err(e) => {
                warn!(track_id, "Replay failed: {}", e);
                vec![]
            }
        };

    let track_id_filter = track_id;
    let stream = async_stream::stream! {
        for msg in replayed {
            if msg.event.track_id == track_id_filter {
                let data = serde_json::to_string(&msg.event).unwrap_or_default();
                yield Ok(Event::default()
                    .event("status_changed")
                    .data(data)
                    .id(msg.id));
            }
        }

        loop {
            match subscriber.next_event(BLOCK_MS).await {
                Ok(Some(msg)) if msg.event.track_id == track_id_filter => {
                    let data = serde_json::to_string(&msg.event).unwrap_or_default();
                    yield Ok(Event::default()
                        .event("status_changed")
                        .data(data)
                        .id(msg.id.clone()));

                    if let Err(e) = subscriber.ack(&msg.id).await {
                        warn!(track_id = track_id_filter, "Failed to ack {}: {}", msg.id, e);
                    }

                    if let Ok(claimed) = subscriber.claim(CLAIM_IDLE_MS).await {
                        for c in claimed {
                            if c.event.track_id == track_id_filter {
                                let data = serde_json::to_string(&c.event).unwrap_or_default();
                                yield Ok(Event::default()
                                    .event("status_changed")
                                    .data(data)
                                    .id(c.id.clone()));
                                let _ = subscriber.ack(&c.id).await;
                            }
                        }
                    }
                }
                Ok(Some(_)) => {}
                Ok(None) => {}
                Err(e) => {
                    warn!(track_id = track_id_filter, "Stream read error: {}", e);
                }
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default()))
}
