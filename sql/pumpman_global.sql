CREATE TABLE IF NOT EXISTS pumpman_global (
    id              BIGSERIAL     PRIMARY KEY,
    owner           BigInt        NOT NULL,
    amount          Decimal       NOT NULL,
    priority_fee    Decimal       NOT NULL,
    slippage        Integer       NOT NULL,
    batch           Integer       NOT NULL,
    speed           Integer       NOT NULL
)
