CREATE TABLE IF NOT EXISTS coins (
    id          BIGSERIAL     PRIMARY KEY,
    mint        TEXT          NOT NULL UNIQUE,
    name        TEXT          NOT NULL,
    symbol      TEXT          NOT NULL,
    twitter     TEXT,
    telegram    TEXT,
    website     TEXT
)
