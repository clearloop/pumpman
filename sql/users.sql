CREATE TABLE IF NOT EXISTS users (
    id          BIGSERIAL     PRIMARY KEY,
    tgid        Int8          NOT NULL    UNIQUE,
    wallet      TEXT          NOT NULL
)
