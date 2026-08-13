use ed25519_dalek::{Signature as SolanaSignature, Verifier, VerifyingKey as SolanaVerifyingKey};
use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use rand_core::{CryptoRng, RngCore};
use sha3::{Digest, Keccak256};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid signature hex encoding")]
    InvalidHex,

    #[error("signature must be 65 bytes (r || s || v)")]
    InvalidSignatureLength,

    #[error("invalid recovery id")]
    InvalidRecoveryId,

    #[error("signature error: {0}")]
    SignatureError(#[from] k256::ecdsa::Error),

    #[error("invalid base58 encoding")]
    InvalidBase58,

    #[error("solana address must decode to a 32-byte public key")]
    InvalidPublicKeyLength,

    #[error("invalid solana signature")]
    InvalidSolanaSignature,
}

/// Generates a random nonce for a sign-in-with-ethereum challenge.
pub fn generate_nonce<R>(rng: &mut R) -> String
where
    R: CryptoRng + RngCore,
{
    let mut bytes = [0u8; 16];
    rng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub fn build_siwe_message(address: &str, nonce: &str, issued_at: &str) -> String {
    format!(
        "Sign in to uChat.\n\nAddress: {address}\nNonce: {nonce}\nIssued At: {issued_at}"
    )
}

fn eth_message_hash(message: &str) -> [u8; 32] {
    let prefixed = format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message);
    Keccak256::digest(prefixed.as_bytes()).into()
}

/// Recovers the checksummed-lowercase `0x`-prefixed Ethereum address that produced
/// `signature_hex` (a `0x`-prefixed 65-byte `r || s || v` hex string) over `message`,
/// using the same scheme as `personal_sign` / `eth_sign`.
pub fn recover_address(message: &str, signature_hex: &str) -> Result<String, Error> {
    let signature_hex = signature_hex.trim_start_matches("0x");
    let sig_bytes = hex::decode(signature_hex).map_err(|_| Error::InvalidHex)?;

    if sig_bytes.len() != 65 {
        return Err(Error::InvalidSignatureLength);
    }

    let (rs, v) = sig_bytes.split_at(64);
    let recovery_byte = match v[0] {
        27 | 28 => v[0] - 27,
        0 | 1 => v[0],
        _ => return Err(Error::InvalidRecoveryId),
    };
    let recovery_id = RecoveryId::from_byte(recovery_byte).ok_or(Error::InvalidRecoveryId)?;

    let signature = Signature::from_slice(rs)?;
    let hash = eth_message_hash(message);
    let verifying_key = VerifyingKey::recover_from_prehash(&hash, &signature, recovery_id)?;

    let encoded_point = verifying_key.to_encoded_point(false);
    let pubkey_bytes = encoded_point.as_bytes();
    let address_hash = Keccak256::digest(&pubkey_bytes[1..]);

    Ok(format!("0x{}", hex::encode(&address_hash[12..])))
}

/// Verifies that `signature_b58` (base58-encoded, 64 bytes) over `message` was produced by
/// the ed25519 keypair whose public key is `address_b58` (base58-encoded, 32 bytes) — the
/// same scheme Solana wallets (Phantom, Solflare, Magic's embedded wallet) use for
/// `signMessage`. Returns `Ok(())` on a valid signature, `Err` otherwise.
pub fn verify_solana_signature(
    address_b58: &str,
    message: &str,
    signature_b58: &str,
) -> Result<(), Error> {
    let pubkey_bytes = bs58::decode(address_b58)
        .into_vec()
        .map_err(|_| Error::InvalidBase58)?;
    let pubkey_bytes: [u8; 32] = pubkey_bytes
        .try_into()
        .map_err(|_| Error::InvalidPublicKeyLength)?;
    let verifying_key =
        SolanaVerifyingKey::from_bytes(&pubkey_bytes).map_err(|_| Error::InvalidPublicKeyLength)?;

    let sig_bytes = bs58::decode(signature_b58)
        .into_vec()
        .map_err(|_| Error::InvalidBase58)?;
    let sig_bytes: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| Error::InvalidSolanaSignature)?;
    let signature = SolanaSignature::from_bytes(&sig_bytes);

    verifying_key
        .verify(message.as_bytes(), &signature)
        .map_err(|_| Error::InvalidSolanaSignature)
}

#[cfg(test)]
mod tests {
    use super::*;
    use k256::ecdsa::SigningKey;

    #[test]
    fn recovers_the_signing_address() {
        let mut rng = crate::new_rng();
        let signing_key = SigningKey::random(&mut rng);
        let verifying_key = VerifyingKey::from(&signing_key);

        let expected_address = {
            let encoded_point = verifying_key.to_encoded_point(false);
            let pubkey_bytes = encoded_point.as_bytes();
            let address_hash = Keccak256::digest(&pubkey_bytes[1..]);
            format!("0x{}", hex::encode(&address_hash[12..]))
        };

        let message = build_siwe_message(&expected_address, "abc123", "2026-08-02T00:00:00Z");
        let hash = eth_message_hash(&message);

        let (signature, recovery_id): (Signature, RecoveryId) =
            signing_key.sign_prehash_recoverable(&hash).unwrap();

        let mut sig_bytes = signature.to_bytes().to_vec();
        sig_bytes.push(recovery_id.to_byte() + 27);
        let signature_hex = format!("0x{}", hex::encode(sig_bytes));

        let recovered = recover_address(&message, &signature_hex).unwrap();
        assert_eq!(recovered, expected_address);
    }

    #[test]
    fn rejects_mismatched_signature() {
        let mut rng = crate::new_rng();
        let signing_key = SigningKey::random(&mut rng);

        let message = "Sign in to uChat.\n\nAddress: 0x0\nNonce: n\nIssued At: t";
        let hash = eth_message_hash(message);
        let (signature, recovery_id): (Signature, RecoveryId) =
            signing_key.sign_prehash_recoverable(&hash).unwrap();

        let mut sig_bytes = signature.to_bytes().to_vec();
        sig_bytes.push(recovery_id.to_byte() + 27);
        let signature_hex = format!("0x{}", hex::encode(sig_bytes));

        let recovered = recover_address("a different message", &signature_hex).unwrap();
        let other_signer = {
            let encoded_point = VerifyingKey::from(&signing_key).to_encoded_point(false);
            let pubkey_bytes = encoded_point.as_bytes();
            let address_hash = Keccak256::digest(&pubkey_bytes[1..]);
            format!("0x{}", hex::encode(&address_hash[12..]))
        };
        assert_ne!(recovered, other_signer);
    }

    #[test]
    fn verifies_solana_signature() {
        use ed25519_dalek::{Signer, SigningKey};

        let mut rng = crate::new_rng();
        let mut secret = [0u8; 32];
        rng.fill_bytes(&mut secret);
        let signing_key = SigningKey::from_bytes(&secret);
        let address = bs58::encode(signing_key.verifying_key().to_bytes()).into_string();

        let message = build_siwe_message(&address, "abc123", "2026-08-12T00:00:00Z");
        let signature = signing_key.sign(message.as_bytes());
        let signature_b58 = bs58::encode(signature.to_bytes()).into_string();

        assert!(verify_solana_signature(&address, &message, &signature_b58).is_ok());
    }

    #[test]
    fn rejects_mismatched_solana_signature() {
        use ed25519_dalek::{Signer, SigningKey};

        let mut rng = crate::new_rng();
        let mut secret = [0u8; 32];
        rng.fill_bytes(&mut secret);
        let signing_key = SigningKey::from_bytes(&secret);
        let address = bs58::encode(signing_key.verifying_key().to_bytes()).into_string();

        let signature = signing_key.sign(b"the signed message");
        let signature_b58 = bs58::encode(signature.to_bytes()).into_string();

        assert!(verify_solana_signature(&address, "a different message", &signature_b58).is_err());
    }
}
