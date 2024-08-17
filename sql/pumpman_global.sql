CREATE TABLE IF NOT EXISTS pumpman_global (
    id          BIGSERIAL     PRIMARY KEY,
    owner       BigInt        NOT NULL,
    batch       BigInt        NOT NULL,
    tx_fee      Decimal       NOT NULL,
    amount      Decimal       NOT NULL,
    speed       BigInt        NOT NULL
)
