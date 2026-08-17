use axum::extract::{Path, Query, State};
use axum::response::sse::{Event, Sse};
use futures::stream::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use crate::adapters::http::router::AppState;
use crate::error::AppError;
use tracing::{info, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SseQuery {
    pub last_event_id: Option<i32>,
}

pub async fn stream_events(
    Path((_movie_id, _scene_id, track_id)): Path<(i32, i32, i32)>,
    Query(query): Query<SseQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let span = Span::current();
    span.set_attribute("track.id", track_id as i64);

    info!(
        track_id,
        last_event_id = ?query.last_event_id,
        "SSE connection opened for license events"
    );

    if let Some(after_id) = query.last_event_id {
        if let Ok(entries) = state.audit_repo.find_after_id(track_id, after_id).await {
            for entry in entries {
                info!(
                    track_id,
                    event_id = entry.id,
                    "Replaying missed event via Last-Event-ID"
                );
            }
        }
    }

    let mut rx = state.event_publisher.subscribe();
    
    let stream = async_stream::stream! {
        while let Ok(event) = rx.recv().await {
            if event.track_id == track_id {
                let data = serde_json::to_string(&event).unwrap_or_default();
                yield Ok(Event::default().event("status_changed").data(data).id(event.license_id.to_string()));
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default()))
}