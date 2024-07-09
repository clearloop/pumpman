CREATE TABLE IF NOT EXISTS coins (
    id          BIGSERIAL     PRIMARY KEY,
    mint        TEXT          NOT NULL UNIQUE
)
