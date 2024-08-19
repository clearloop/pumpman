CREATE TABLE IF NOT EXISTS users (
    id          BIGSERIAL     PRIMARY KEY,
    created_at  Date          NOT NULL,
    tgid        Int8          NOT NULL    UNIQUE,
    wallet      TEXT          NOT NULL
)
