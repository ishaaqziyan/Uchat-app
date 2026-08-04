ALTER TABLE users ADD COLUMN eth_address TEXT UNIQUE;

CREATE TABLE wallet_nonces (
    id UUID NOT NULL PRIMARY KEY,
    eth_address TEXT NOT NULL,
    nonce TEXT NOT NULL,
    message TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    consumed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX wallet_nonces_address_idx ON wallet_nonces (eth_address);
