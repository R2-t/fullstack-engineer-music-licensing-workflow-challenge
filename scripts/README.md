# SSE Monitor

Observer + event trigger for the Music Licensing backend's SSE endpoint
(`/movies/{movie_id}/scenes/{scene_id}/tracks/{track_id}/licenses/events`).

## Setup

Requires [uv](https://docs.astral.sh/uv/):

```sh
uv sync
```

## Usage

```sh
uv run python sse_monitor.py
```

The script auto-creates a movie, scene, track, and license, connects to the
SSE endpoint, then drives the license workflow (`DRAFT -> NEGOTIATING ->
(REJECTED <-> NEGOTIATING)* -> APPROVED`) to generate live events. It prints a
JSON report:

```json
{
  "status": "success",
  "event_count": 4,
  "sample_updates": [ ...last 5 events... ],
  "errors_encountered": null
}
```

## Environment variables

| Variable | Default | Purpose |
|---|---|---|
| `SSE_BASE_URL` | `http://localhost:8080` | Base URL for API + SSE |
| `SSE_MOVIE_ID` | (auto-create) | Reuse existing movie |
| `SSE_SCENE_ID` | (auto-create) | Reuse existing scene |
| `SSE_TRACK_ID` | (auto-create) | Reuse existing track |
| `SSE_MAX_EVENTS` | `20` | Stop after N events |
| `SSE_TIMEOUT_SECONDS` | `30` | Hard overall timeout |
| `SSE_CONNECT_TIMEOUT` | `5` | Per-request connect timeout |
| `SSE_IDLE_WINDOW` | `15` | Max seconds without data before closing |

Example with existing resources:

```sh
SSE_MOVIE_ID=1 SSE_SCENE_ID=1 SSE_TRACK_ID=1 SSE_MAX_EVENTS=4 uv run python sse_monitor.py
```
