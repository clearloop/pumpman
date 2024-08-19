CREATE TABLE IF NOT EXISTS pumpmen (
    id              BIGSERIAL     PRIMARY KEY,
    mint            TEXT          NOT NULL,
    owner           BigInt        NOT NULL,
    wallet          TEXT,
    created_at      Date          NOT NULL,
    active          Boolean       NOT NULL,
    amount          Decimal       NOT NULL,
    priority_fee    Decimal       NOT NULL,
    batch           Integer       NOT NULL,
    speed           Integer       NOT NULL,
    bumps           BigInt        NOT NULL,
    charged         Decimal       NOT NULL
)
