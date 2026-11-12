CREATE TABLE beans (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    origin TEXT NOT NULL,
    roaster TEXT NOT NULL,
    process TEXT NOT NULL,
    elevation TEXT NOT NULL,
    tasting_notes TEXT NOT NULL,
    image_key TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX beans_user_id_idx ON beans (user_id, id);

CREATE TABLE uploads (
    key TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    in_processing BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX beans_user_image_key_uidx ON beans (user_id, image_key);
