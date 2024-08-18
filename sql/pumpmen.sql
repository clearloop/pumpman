CREATE TABLE IF NOT EXISTS pumpmen (
    id          BIGSERIAL     PRIMARY KEY,
    active      Boolean       NOT NULL,
    created_at  Date          NOT NULL,
    owner       BigInt        NOT NULL,
    mint        TEXT          NOT NULL,
    tx_fee      Decimal       NOT NULL,
    amount      Decimal       NOT NULL,
    batch       Integer       NOT NULL,
    speed       Integer       NOT NULL,
    bumps       BigInt        NOT NULL,
    wallet      TEXT
)
