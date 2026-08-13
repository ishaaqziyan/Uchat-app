DROP INDEX wallet_nonces_solana_address_idx;
ALTER TABLE wallet_nonces DROP CONSTRAINT wallet_nonces_address_chk;
ALTER TABLE wallet_nonces DROP COLUMN solana_address;
ALTER TABLE wallet_nonces ALTER COLUMN eth_address SET NOT NULL;

ALTER TABLE users DROP COLUMN solana_address;
