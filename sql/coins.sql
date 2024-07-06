CREATE TABLE IF NOT EXISTS coins (
    description   TEXT,
    image         TEXT,
    mint          TEXT    NOT NULL    PRIMARY KEY,
    name          TEXT    NOT NULL,
    symbol        TEXT    NOT NULL,
    telegram      TEXT,
    twitter       TEXT,
    website       TEXT,
    created_on    TEXT
)
