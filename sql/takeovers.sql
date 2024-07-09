CREATE TABLE IF NOT EXISTS takeovers (
  id          BIGSERIAL     PRIMARY KEY,
  banner      TEXT,
  mint        TEXT          NOT NULL    REFERENCES coins(mint),
  admin       TEXT          NOT NULL    REFERENCES users(tgid),
  telegram    TEXT          NOT NULL,
  twitter     TEXT,
  website     TEXT
)
