CREATE TABLE beans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    origin TEXT NOT NULL,
    roaster TEXT NOT NULL,
    process TEXT NOT NULL,
    elevation TEXT NOT NULL,
    tasting_notes TEXT NOT NULL,
    image_key TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX beans_user_id_idx ON beans (user_id, id);

CREATE TABLE uploads (
    key TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    in_processing INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX beans_user_image_key_uidx ON beans (user_id, image_key);
