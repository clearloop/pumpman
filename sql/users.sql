CREATE TABLE IF NOT EXISTS users (
    id          BIGSERIAL     PRIMARY KEY,
    tgid        TEXT          NOT NULL    UNIQUE,
    credits     Int8          NOT NULL    DEFAULT 3
)
