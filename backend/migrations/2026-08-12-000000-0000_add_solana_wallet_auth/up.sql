ALTER TABLE users ADD COLUMN solana_address TEXT UNIQUE;

ALTER TABLE wallet_nonces ALTER COLUMN eth_address DROP NOT NULL;
ALTER TABLE wallet_nonces ADD COLUMN solana_address TEXT;
ALTER TABLE wallet_nonces ADD CONSTRAINT wallet_nonces_address_chk
    CHECK ((eth_address IS NOT NULL) <> (solana_address IS NOT NULL));

CREATE INDEX wallet_nonces_solana_address_idx ON wallet_nonces (solana_address);
