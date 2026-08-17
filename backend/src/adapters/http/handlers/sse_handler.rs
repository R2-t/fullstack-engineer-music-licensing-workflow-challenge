use axum::extract::{Path, State};
use axum::response::sse::{Event, Sse};
use futures::stream::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::StreamExt;
use crate::adapters::http::router::AppState;
use crate::error::AppError;

pub async fn stream_events(
    Path((_movie_id, _scene_id, track_id)): Path<(i32, i32, i32)>,
    State(state): State<Arc<AppState>>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let mut rx = state.event_publisher.subscribe();
    
    let stream = async_stream::stream! {
        while let Ok(event) = rx.recv().await {
            if event.track_id == track_id {
                let data = serde_json::to_string(&event).unwrap_or_default();
                yield Ok(Event::default().event("status_changed").data(data));
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default()))
}