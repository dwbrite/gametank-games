CREATE TABLE IF NOT EXISTS game_entries (
    game_id UUID PRIMARY KEY,
    game_name TEXT NOT NULL,
    description TEXT NOT NULL,
    author UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    game_rom BYTEA NOT NULL,
    public_access BOOL NOT NULL DEFAULT false
);

CREATE TABLE user_login (
    user_id UUID PRIMARY KEY,
    last_login TIMESTAMP NOT NULL
);