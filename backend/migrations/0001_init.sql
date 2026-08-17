-- 📍 Movie Table
CREATE TABLE IF NOT EXISTS movies (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    release_date DATE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- 📍 Movie Scene Table
CREATE TABLE IF NOT EXISTS scenes (
    id SERIAL PRIMARY KEY,
    movie_id INT NOT NULL REFERENCES movies(id) ON DELETE CASCADE,
    scene_number SMALLINT NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- 📍 Track Table (Denormalized with song metadata)
CREATE TABLE IF NOT EXISTS tracks (
    id SERIAL PRIMARY KEY,
    scene_id INT NOT NULL REFERENCES scenes(id) ON DELETE CASCADE,
    track_order SMALLINT DEFAULT 0,
    name VARCHAR(255) DEFAULT 'Unnamed Track',
    song_title VARCHAR(255) DEFAULT 'Untitled',
    song_artist VARCHAR(150),
    song_label VARCHAR(250),
    duration_sec_start INTEGER NOT NULL DEFAULT 0,
    duration_sec_end INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- 📍 License Tracking Table
CREATE TABLE IF NOT EXISTS licenses (
    id SERIAL PRIMARY KEY,
    track_id INT NOT NULL UNIQUE REFERENCES tracks(id) ON DELETE CASCADE,
    label_name VARCHAR(250),
    artist_name VARCHAR(150),
    status TEXT DEFAULT 'DRAFT' NOT NULL,
    negotiation_notes JSONB DEFAULT '{}' NOT NULL,
    last_updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- 📍 License Audit Log
CREATE TABLE IF NOT EXISTS license_audit_log (
    id SERIAL PRIMARY KEY,
    track_id INT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    license_id INT NOT NULL REFERENCES licenses(id) ON DELETE CASCADE,
    user_email VARCHAR(150) NOT NULL,
    old_status TEXT,
    new_status TEXT NOT NULL,
    changed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_scenes_movie ON scenes(movie_id);
CREATE INDEX IF NOT EXISTS idx_tracks_scene ON tracks(scene_id);
CREATE INDEX IF NOT EXISTS idx_license_track ON licenses(track_id);
CREATE INDEX IF NOT EXISTS idx_audit_license ON license_audit_log(license_id);
CREATE INDEX IF NOT EXISTS idx_audit_track ON license_audit_log(track_id);