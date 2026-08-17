# 🎬 Movie Music Licensing System - Database Schema (PostgreSQL)

Here are the **normalized table schemas** using standard Postgres SQL, designed with `JSONB` fields
to store license metadata flexibly while maintaining strong relationships through foreign keys. All
tables use UUIDs where appropriate for scalability across environments and projects.

## 1. Core Tables & Relationships Overview
```sql
-- 📍 Movie Table (Tracks belong here too)
CREATE TABLE movies (
    id SERIAL PRIMARY KEY,                      -- Auto-increment ID or UUID: uuid-v7()
    title VARCHAR(255),                         -- e.g., "The Last Laugh"
    release_date DATE,                          -- YYYY-MM-DD format for sorting/restriction
filters

    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 📍 Movie Scene Table (Scene Order Matters!)
CREATE TABLE scenes (
    id SERIAL PRIMARY KEY,                      -- FK to movies or separate ID per movie?
    movie_id INT NOT NULL REFERENCES movies(id), -- Link back to parent
    scene_number SMALLINT NOT NULL DEFAULT 0,   -- e.g., "ACT_01" -> Scene #2 of that film

    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 📍 Track Table (Audio/Content Segment)
CREATE TABLE tracks (
    id SERIAL PRIMARY KEY,                      -- FK to scenes or UUID? Use uuid for global
uniqueness.

    scene_id INT NOT NULL REFERENCES scenes(id),  -- Link back to the movie section it lives in

    track_order SMALLINT DEFAULT 0,             -- Which song is this? e.g., "1 of X"

    name VARCHAR(255) DEFAULT 'Unnamed Track',   -- Internal working title
    duration_sec_start INTEGER NOT NULL DEFAULT 0,     -- Start time (sec from beginning of scene
audio?)
    duration_sec_end INTEGER NOT NULL                   -- End time in seconds

    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 📍 License Tracking Table (State Machine per song)
CREATE TABLE licenses (
    id SERIAL PRIMARY KEY,                      -- PK for licensing workflow tracking. Use UUID if
you prefer uniqueness across all projects.

    track_id INT NOT NULL REFERENCES tracks(id),  -- FK links to a specific audio segment

    label_name VARCHAR(250),                    -- e.g., "Warner Bros." or "Sony Music"
    artist_name VARCHAR(150),                   -- Who are we negotiating with?

    status TEXT DEFAULT 'DRAFT',                -- State: DRAFT → NEGOTIATING → APPROVED/REJECTED

    negotiation_notes JSONB,                    -- Flexible metadata (who was contacted at what
step)
    last_updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP  -- When did this change happen?

);
```

## 🚀 Workflow Status Mapping & Validation
The license status can only be transitioned in specific sequences to prevent invalid states. You’ll
define these rules:

| Initial State | Allowed Transitions          | Notes                                  |
| ------------- | ---------------------------- | -------------------------------------- |
| `DRAFT`       | → `NEGOTIATING`              | When licensing request is created      |
| `NEGOTIATING` | → `APPROVED`, **→ REJECTED** | Agreement reached or negotiation fails |
| `REJECTED`    | → `NEGOTIATING`              | Retry failed negotiation               |

> You can enforce this logic in your API layer using a service class like `WorkflowValidator`. The
database doesn't block out-of-order transitions unless you add application checks. This allows
flexibility for manual overrides (e.g., "Approve despite label concerns").

---

## 🔌 Additional Supporting Tables
Use these auxiliary tables if needed:

### 2. Notification / Event Tracking Table
For audit trails of who changed what and when:

```sql
CREATE TABLE license_audit_log (
    id SERIAL PRIMARY KEY,

    track_id INT NOT NULL REFERENCES tracks(id), -- Which song's status?

    user_email VARCHAR(150),                     -- Who initiated change?
    old_status TEXT DEFAULT 'DRAFT',             -- Previous state value before transition

    new_status TEXT DEFAULT 'NEGOTIATING'        -- New state after update
);

CREATE INDEX idx_license_audit_track ON license_audit_log(track_id);
```

### 3. Real-time Subscription Table (For WebSocket clients)
Used by subscribers to receive updates when a track's licensing changes:

```sql
CREATE TABLE client_subscriptions (
    id SERIAL PRIMARY KEY,                         -- PK

    client_name VARCHAR(100),                      -- Who is subscribed? e.g., "Alice" or internal
system ID

    subscription_start TIMESTAMP DEFAULT CURRENT_TIMESTAMP  -- When did they join this stream?

);

-- Optional: Track which subscribers are listening to a specific track
CREATE INDEX idx_subs_track ON client_subscriptions(track_id) WHERE status='ACTIVE';
```

---

## Design decisions

The tracks table now includes three denormalized song-identity fields: song_title, song_artist,
and song_label. This decision is intentional and carries the following rationale:

- Single-read optimization. The dominant API operation is GET .../tracks/{trackId} — the consumer
requests the track and its licensing status in one round-trip. By co-locating the song fields
with the track row, this is a single-DB-query operation. Introducing a separate songs table
would require an additional join on every track read, adding latency for a pattern that
accounts for the majority of API traffic.
- Placement-bound metadata. Each scene placement has its own start/end window and its own
negotiation lifecycle. Song title, artist, and label are naturally co-located with the
placement they describe. A standalone songs catalog would model a global song identity that
this challenge's data shape doesn't require, adding indirection without functional benefit.
- Bounded duplication is acceptable. At the scale of this system, the same musical work appearing
in multiple scenes is expected to be duplicated rather than shared via a catalog. The per-track
license workflow (DRAFT → NEGOTIATING → {APPROVED|REJECTED}) is the integrity boundary; the
song fields are descriptive metadata about that specific placement, not a reference to a global
work.
- Simplicity over premature generalization. Adding a songs table and song_id FK assumes future
need for song catalog queries ("find all tracks using Song X") that the current requirements
don't mandate. Keeping the fields on tracks keeps the schema minimal and the migration path
straightforward. A songs catalog can be added later if the system evolves to need it.
