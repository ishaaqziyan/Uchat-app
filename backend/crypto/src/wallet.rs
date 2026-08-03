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
}
