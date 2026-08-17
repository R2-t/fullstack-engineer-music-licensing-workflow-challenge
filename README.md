# Music Licensing Workflow

A backend system for managing the music licensing process of movie scenes, built with **Rust + Axum** and a **hexagonal (ports & adapters) architecture**. It exposes a REST API for movies, scenes, tracks and their per-track licenses, and delivers live license-status updates to clients over **Server-Sent Events (SSE)** powered by **Redis Streams**.

The license state machine (`DRAFT → NEGOTIATING → {APPROVED, REJECTED}` with `REJECTED → NEGOTIATING`) is enforced in the domain layer, persisted in **PostgreSQL**, and every transition is published as an event for real-time consumption.

## Table of Contents

- [Features](#features)
- [Architecture](#architecture)
- [Tech Stack](#tech-stack)
- [Key Decisions](#key-decisions)
- [Project Structure](#project-structure)
- [Getting Started](#getting-started)
- [Environment Variables](#environment-variables)
- [API Reference](#api-reference)
- [License Workflow](#license-workflow)
- [Real-time Events (SSE)](#real-time-events-sse)
- [Testing & CI](#testing--ci)
- [References](#references)

## Features

- **REST API** for CRUD operations on movies, scenes, and tracks, plus license initiation and status transitions.
- **Stateful license workflow** validated by a dedicated domain service (`WorkflowValidator`).
- **Audit trail** of every status change stored in `license_audit_log`.
- **Real-time updates** over SSE, with replay support (`Last-Event-ID`) and consumer-group message acknowledgment/claiming via Redis Streams.
- **Observability** with OpenTelemetry traces exported to Jaeger.
- **Containerized** via Docker Compose (PostgreSQL, Redis, Jaeger, backend).
- **CI pipeline** via GitHub Actions (fmt, clippy, tests, image build/push).
- **Observer script** (`scripts/sse_monitor.py`) to drive the workflow and validate the SSE stream end-to-end.

## Architecture

The codebase follows a hexagonal layout, separating the core domain from external concerns:

```
┌──────────────────────────────────────────────────────────────┐
│                        HTTP (Axum)                          │
│   handlers: movie, scene, track, license, sse (events)      │
└──────────────────────────────┬───────────────────────────────┘
                               │
┌──────────────────────────────▼───────────────────────────────┐
│                      Application Services                   │
│            LicenseService, MovieService, SceneService,       │
│                       TrackService                           │
└──────────────────────────────┬───────────────────────────────┘
                               │
┌──────────────────────────────▼───────────────────────────────┐
│                     Domain + Ports (traits)                 │
│   entities, WorkflowValidator, repository & event ports     │
└──────┬───────────────────────────────────────┬───────────────┘
       │                                       │
┌──────▼───────────────┐            ┌──────────▼───────────────┐
│  Adapters: Database  │            │  Adapters: Realtime      │
│  sqlx / PostgreSQL   │            │  Redis Streams hub       │
└──────────────────────┘            └──────────────────────────┘
```

### Real-time data flow

1. A client calls `PATCH .../licenses/status-transition`.
2. `LicenseService` validates the transition against the workflow rules and persists the new status (plus an audit entry) in PostgreSQL.
3. The service publishes a `status_changed` event to the Redis Stream `licenses:events`.
4. Each SSE connection joins the consumer group `events-grp`. The SSE handler reads new entries, acknowledges them, and optionally reclaims abandoned ones via `XAUTOCLAIM`.
5. Events are streamed to the browser/agent as SSE `event: status_changed`, carrying the event payload and its Redis Stream `id` as the SSE `id` (used for replay).

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (edition 2021) |
| Web framework | Axum 0.7 |
| Async runtime | Tokio |
| Database | PostgreSQL 16 via `sqlx` 0.8 |
| Real-time | Redis 7 Streams (consumer groups) |
| Auth | JSON Web Tokens (`jsonwebtoken`) — implemented, disabled by default |
| Observability | OpenTelemetry + Jaeger (`tracing-opentelemetry`) |
| Containerization | Docker / Docker Compose |
| CI | GitHub Actions |
| Observer tooling | Python 3.10+, `requests`, managed with `uv` |

## Key Decisions

- **Why Rust + Axum.** Rust offers memory safety and high concurrency; Axum provides strong type safety, composable middleware, and first-class SSE support, fitting a small team and a system where correctness of the workflow matters.
- **Why REST only.** The domain is a small set of CRUD resources plus explicit status transitions. There is no need for the query flexibility GraphQL provides. REST keeps the API predictable and trivially consumable by any SSE/HTTP client.
- **Why SSE + Redis Streams over WebSockets / Kafka.**
  - License updates flow in a single direction (server → client), which is exactly the SSE use case — no client→server messaging is required.
  - Redis Streams provide a **durable, replayable event log**: a client reconnecting can send `Last-Event-ID` to receive only missed events.
  - Consumer groups add acknowledgment and **orphan-message claiming**, so a failed consumer does not permanently lose events.
  - Compared to Kafka, Redis is far lighter to operate for this workload while still giving ordering and at-least-once delivery semantics.
- **Denormalized song fields on `tracks`.** Song title/artist/label and the placement start/end windows live on the track row so `GET .../tracks/{id}` is a single query. See [`ARCHITECTURE.md`](ARCHITECTURE.md) for the full rationale.
- **Auth implemented but disabled by default.** The JWT middleware exists in `backend/src/adapters/http/auth.rs` but is commented out in `router.rs` so the API is easy to exercise for development/testing. Re-enable it by uncommenting the `.layer(...)` block and setting a real `JWT_SECRET`.
- **Idempotent migrations.** Migration SQL uses `IF NOT EXISTS` and is split per-statement for PostgreSQL compatibility, so startup is repeatable.
- **Observability first.** Every repository and Redis operation is instrumented with `tracing::instrument`, exporting spans to Jaeger for end-to-end request tracing.

## Project Structure

```
.
├── .github/
│   ├── workflows/
│   │   └── ci.yml              # fmt, clippy, tests, Docker build/push
│   └── PULL_REQUEST_TEMPLATE.md
├── ARCHITECTURE.md              # Database schema & denormalization rationale
├── openapi.yaml                 # OpenAPI 3.0 specification
├── compose.yml                  # Postgres, Redis, Jaeger, backend
├── backend/
│   ├── .env.example
│   ├── Cargo.toml / Cargo.lock
│   ├── Dockerfile               # multi-stage build (cargo-chef)
│   ├── migrations/0001_init.sql
│   ├── src/
│   │   ├── main.rs              # startup, migration runner, wiring
│   │   ├── lib.rs
│   │   ├── config.rs            # env-based configuration
│   │   ├── error.rs             # unified AppError
│   │   ├── domain/              # entities, LicenseStatus, WorkflowValidator
│   │   ├── ports/               # repository & event traits (interfaces)
│   │   ├── application/
│   │   │   └── services/        # LicenseService, MovieService, SceneService, TrackService
│   │   ├── infrastructure/      # tracing setup
│   │   └── adapters/
│   │       ├── db/              # sqlx Postgres repository implementations
│   │       ├── http/            # handlers, router, DTOs, JWT auth
│   │       └── realtime.rs      # Redis Streams hub & subscriber
│   └── tests/integration_test.rs
└── scripts/
    ├── sse_monitor.py           # observer that drives workflow + reads SSE
    ├── pyproject.toml           # uv project (requests)
    ├── README.md
    └── uv.lock
```

## Getting Started

### Prerequisites

- **Docker** + **Docker Compose** (easiest path).
- Alternatively **Rust toolchain** (stable) to run the backend locally.
- **uv** (only if you want to run the SSE observer script).

### Option 1 — Full stack with Docker Compose

```bash
docker compose up --build
```

This starts:

| Service | Address |
|---------|---------|
| Backend API | http://localhost:8080 |
| Jaeger UI | http://localhost:16686 |
| PostgreSQL | localhost:5432 |
| Redis | localhost:6379 |

### Option 2 — Run the backend locally

```bash
cd backend
cp .env.example .env       # adjust values if needed

# 1. Start only the data stores (or use your own Postgres/Redis)
docker compose up -d db redis jaeger

# 2. Run the server (applies migrations on startup)
cargo run
```

### Option 3 — Validate real-time events with the observer

```bash
cd scripts
uv sync
uv run python sse_monitor.py
```

With no configuration, the script auto-creates a movie, scene, track, and license, connects to the SSE endpoint, drives the license through `DRAFT → NEGOTIATING → REJECTED → NEGOTIATING → APPROVED`, and prints a JSON report (`status`, `event_count`, `sample_updates`, `errors_encountered`). See [`scripts/README.md`](scripts/README.md) for all options.

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `postgres://postgres:password@localhost:5432/licensing_db` | PostgreSQL connection URL |
| `REDIS_URL` | `redis://localhost:6379` | Redis connection URL |
| `PORT` | `8080` | HTTP listen port |
| `HOST` | `0.0.0.0` | HTTP bind address |
| `JWT_SECRET` | `secret_key_change_me` | Secret used to sign/verify JWTs |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | `http://localhost:4317` | OTLP endpoint for Jaeger |
| `RUST_LOG` | `info,music_licensing_backend=debug,sqlx=warn,tower_http=debug` | Log/trace filter |

## API Reference

All routes are served under the base URL `http://localhost:8080`.

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/movies` | List movies |
| `POST` | `/movies` | Create a movie |
| `GET` | `/movies/:id` | Get a movie |
| `POST` | `/movies/:id/scenes` | Create a scene for a movie |
| `GET` | `/movies/:id/scenes` | List scenes for a movie |
| `GET` | `/movies/:movie_id/scenes/:scene_id` | Get a scene |
| `POST` | `/movies/:movie_id/scenes/:scene_id/tracks` | Create a track (with song) for a scene |
| `GET` | `/movies/:movie_id/scenes/:scene_id/tracks` | List tracks for a scene |
| `GET` | `/movies/:movie_id/scenes/:scene_id/tracks/:track_id` | Get a track |
| `PATCH` | `/movies/:movie_id/scenes/:scene_id/tracks/:track_id` | Update a track |
| `DELETE` | `/movies/:movie_id/scenes/:scene_id/tracks/:track_id` | Delete a track |
| `POST` | `/movies/:movie_id/scenes/:scene_id/tracks/:track_id/licenses` | Initiate a license for a track |
| `GET` | `/movies/:movie_id/scenes/:scene_id/tracks/:track_id/licenses` | Get the license status for a track |
| `PATCH` | `/movies/:movie_id/scenes/:scene_id/tracks/:track_id/licenses/status-transition` | Transition the license status |
| `GET` | `/movies/:movie_id/scenes/:scene_id/tracks/:track_id/licenses/events` | Stream license status-change events (SSE) |

See [`openapi.yaml`](openapi.yaml) for the full machine-readable specification.

## License Workflow

A license begins in `DRAFT`. The allowed transitions are:

| From | To |
|------|----|
| `DRAFT` | `NEGOTIATING` |
| `NEGOTIATING` | `APPROVED` |
| `NEGOTIATING` | `REJECTED` |
| `REJECTED` | `NEGOTIATING` |

Any other transition (e.g., `DRAFT → APPROVED` or `APPROVED → *`) is rejected by `WorkflowValidator` with an `InvalidTransition` error. `APPROVED` is a terminal state.

```
DRAFT ──────► NEGOTIATING ──────► APPROVED
                 ▲   │
                 │   └─────────► REJECTED
                 └──────────────┘
```

## Real-time Events (SSE)

Subscribe to a track's license events:

```bash
curl -N \
  -H "Accept: text/event-stream" \
  http://localhost:8080/movies/1/scenes/1/tracks/1/licenses/events
```

Each event looks like:

```
event: status_changed
id: 1710000000123-5
data: {"stream_id":"1710000000123-5","track_id":1,"license_id":2,"previous_status":"DRAFT","status":"NEGOTIATING","changed_by":"user@example.com","timestamp":"2026-08-17T03:56:00.316405349Z"}
```

To resume after a disconnect, send the last received SSE `id` as the `Last-Event-ID` header (or a `?last_event_id=` query parameter); the server replays missed events from the stream.

## Testing & CI

```bash
cd backend
cargo fmt --all -- --check   # formatting
cargo clippy --all-targets -- -D warnings   # linting
cargo test                   # unit + integration tests
```

The GitHub Actions workflow (`.github/workflows/ci.yml`) runs the above on every push/PR to `main`, and on `main` builds and pushes the backend image to GHCR.

## References

- [`ARCHITECTURE.md`](ARCHITECTURE.md) — database schema and denormalization design decisions.
- [`openapi.yaml`](openapi.yaml) — OpenAPI 3.0 API specification.
- [`scripts/README.md`](scripts/README.md) — SSE observer usage and configuration.
- [`LICENSE`](LICENSE) — project license.
