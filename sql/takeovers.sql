CREATE TABLE IF NOT EXISTS takeovers (
  id          BIGSERIAL     PRIMARY KEY,
  banner      TEXT,
  mint        TEXT          NOT NULL    REFERENCES coins(mint),
  proposer    TEXT          NOT NULL,
  telegram    TEXT          NOT NULL,
  twitter     TEXT,
  website     TEXT
)
