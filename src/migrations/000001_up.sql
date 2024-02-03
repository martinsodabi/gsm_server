-- FILS SQLITE DB ATLAS MIGRATION COMMAND
-- atlas schema apply --url "sqlite://gsm.db"  --to file://src/migrations/000001_up.sql --dev-url "sqlite://atlas.db"
-- atlas schema apply --url "sqlite://gsm.db"  --to file://src/migrations/000001_down.sql --dev-url "sqlite://atlas.db"

-- TURSO DB ATLAS MIGRATION COMMAND
-- atlas schema apply --env turso  --to file://src/migrations/000001_up.sql --dev-url "sqlite://atlas.db"
-- atlas schema apply --env turso  --to file://src/migrations/000001_down.sql --dev-url "sqlite://atlas.db"

CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    pid BLOB(16) UNIQUE NOT NULL CHECK(length(pid) = 16),
    email TEXT(255) UNIQUE NOT NULL,
    password TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER
);

CREATE TABLE IF NOT EXISTS profiles (
    id INTEGER PRIMARY KEY,
    pid BLOB(16) UNIQUE NOT NULL CHECK(length(pid) = 16),
    user_id INTEGER UNIQUE NOT NULL,
    birth_date INTEGER NOT NULL,
    first_name TEXT(255) NOT NULL,
    last_name TEXT(255) NOT NULL,
    location TEXT(255),
    video_path TEXT(255), -- Duplicating this here due to frequent reads
    is_visible BOOLEAN NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
