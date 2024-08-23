CREATE TABLE IF NOT EXISTS takeovers (
  id          BIGSERIAL     PRIMARY KEY,
  banner      TEXT,
  mint        TEXT          NOT NULL    REFERENCES coins(mint)    UNIQUE,
  admin       Int8          NOT NULL    REFERENCES users(tgid)    UNIQUE,
  telegram    TEXT          NOT NULL,
  twitter     TEXT,
  website     TEXT
)
